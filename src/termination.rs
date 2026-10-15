//! Austritt & Auflösung: «LEG auflösen» und «Messpunkte austreten lassen».
//!
//! Beide Aktionen lösen in LEGhub denselben Ablauf aus wie das Dashboard: Meldung an den
//! Netzbetreiber (bzw. innosolv), danach Speichern in LEGhub. Anders als beim Löschen gibt es
//! **keinen Probelauf** – das Enddatum ist danach unveränderlich. Deshalb prüft das Tool die
//! Eingaben vorab selbst und zeigt im Bestätigungs-Popup alles Wichtige noch einmal an.
//!
//! Server-Seite: leg-Servlets `supportDeactivateLeg` / `supportDeactivateMeasuringPoint`.
//! Die Fehler kommen als Codes (`errors[].code`) mit englischem Text; hier werden die bekannten
//! Codes auf Deutsch übersetzt, unbekannte zeigen den Server-Text.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use dioxus::prelude::*;
use serde::Deserialize;

use crate::{error, key_error, ActionResult, Env, PendingRequest, RequestBody, ResultKind};

/// Lässt die Antwort nicht beliebig lange auf sich warten: die Meldung an den Netzbetreiber kann dauern,
/// aber ein hängender Aufruf soll irgendwann als «möglicherweise ausgeführt» gemeldet werden.
const TIMEOUT_SECS: u64 = 180;

/// Austritt / Auflösung? (Diese Aktionen haben keinen Probelauf.)
pub fn is_termination_action(path_suffix: &str) -> bool {
    matches!(path_suffix, "supportDeactivateLeg" | "supportDeactivateMeasuringPoint")
}

/// Felder, die als Datumsauswahl angezeigt werden.
pub fn is_date_field(key: &str) -> bool {
    key == "endingDate"
}

/// Was im Bestätigungs-Popup und in der Zusammenfassung angezeigt wird.
#[derive(Clone, PartialEq)]
pub struct TerminationInfo {
    /// `true` = LEG auflösen, `false` = Messpunkte austreten lassen.
    pub is_leg: bool,
    pub leg_id: String,
    /// JJJJ-MM-TT, wie gesendet.
    pub ending_date: String,
    pub measuring_points: Vec<String>,
    pub storage_only: bool,
}

impl TerminationInfo {
    /// Hinweise zum Datum, die vor dem Absenden deutlich angezeigt werden (leer = unauffällig).
    pub fn date_warnings(&self) -> Vec<String> {
        date_warnings(&self.ending_date, &today())
    }
}

// ----------------------------------------------------------------------------
// Antwort des Servers
// ----------------------------------------------------------------------------

#[derive(Clone, PartialEq, Deserialize)]
pub struct MutationError {
    #[serde(default)]
    pub code: String,
    /// AJILA (Prüfung in LEGhub), VNB (Netzbetreiber) oder STORAGE (LEGhub-Datenbank).
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub retryable: bool,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub params: HashMap<String, Option<String>>,
}

#[derive(Deserialize)]
struct LegResult {
    status: String,
    #[serde(default)]
    errors: Vec<MutationError>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MeasuringPointResult {
    #[serde(default)]
    measuring_point_number: Option<String>,
    status: String,
    #[serde(default)]
    errors: Vec<MutationError>,
}

#[derive(Deserialize)]
struct MeasuringPointResponse {
    results: Vec<MeasuringPointResult>,
}

#[derive(Deserialize)]
struct ProblemBody {
    detail: Option<String>,
}

/// Ergebnis für die LEG bzw. für einen Messpunkt.
#[derive(Clone, PartialEq)]
pub struct ReportRow {
    /// Messpunkt-Nummer; `None` bei der Auflösung einer LEG.
    pub measuring_point: Option<String>,
    pub ok: bool,
    pub errors: Vec<MutationError>,
}

impl ReportRow {
    /// Lohnt sich ein erneuter Versuch? Nur wenn jeder Fehler das erlaubt (wie die `retryable`-Regel des Servers).
    pub fn retryable(&self) -> bool {
        !self.ok && !self.errors.is_empty() && self.errors.iter().all(|e| e.retryable)
    }
}

/// Auswertung einer Antwort mit Ergebnis (Status 200).
#[derive(Clone, PartialEq)]
pub struct TerminationReport {
    pub info: TerminationInfo,
    pub rows: Vec<ReportRow>,
}

impl TerminationReport {
    pub fn all_ok(&self) -> bool {
        self.rows.iter().all(|r| r.ok)
    }

    pub fn any_ok(&self) -> bool {
        self.rows.iter().any(|r| r.ok)
    }

    pub fn any_retryable(&self) -> bool {
        self.rows.iter().any(ReportRow::retryable)
    }

    pub fn kind(&self) -> ResultKind {
        if self.all_ok() {
            ResultKind::Success
        } else if self.any_ok() {
            ResultKind::Warning
        } else {
            ResultKind::Error
        }
    }

    /// Klartext zum Kopieren ins Ticket.
    pub fn to_ticket_text(&self, env: Env) -> String {
        let info = &self.info;
        let what = if info.is_leg { "LEG auflösen" } else { "Messpunkte austreten lassen" };
        let mut lines = vec![
            format!("LEGhub Support-Tool: {what} ({})", env.label()),
            format!("LEG: {}", info.leg_id),
            format!("Enddatum: {}", swiss_date(&info.ending_date)),
            format!(
                "Netzbetreiber informiert: {}",
                if info.storage_only { "nein (nur in LEGhub gespeichert, separat melden)" } else { "ja" }
            ),
        ];
        for row in &self.rows {
            let subject = row.measuring_point.as_deref().map(|nr| format!("Messpunkt {nr}")).unwrap_or_else(|| "LEG".into());
            if row.ok {
                lines.push(format!("{subject}: erledigt"));
            } else {
                let reasons: Vec<String> = row.errors.iter().map(|e| format!("{} ({})", german_message(e), e.code)).collect();
                lines.push(format!("{subject}: FEHLER – {}", reasons.join("; ")));
            }
        }
        lines.join("\n")
    }
}

pub enum TerminationOutcome {
    Report(TerminationReport),
    Failed {
        result: ActionResult,
        /// Ein erneuter Versuch ist sinnvoll (z. B. Antwort ausgeblieben).
        can_retry: bool,
    },
}

/// Sendet Austritt bzw. Auflösung und wertet die Antwort aus.
pub async fn send_termination_request(req: &PendingRequest) -> TerminationOutcome {
    let Some(info) = req.termination.clone() else {
        return TerminationOutcome::Failed { result: crate::generic_error(), can_retry: false };
    };
    let RequestBody::Form(form) = &req.body else {
        return TerminationOutcome::Failed { result: crate::generic_error(), can_retry: false };
    };
    let client = match reqwest::Client::builder().timeout(std::time::Duration::from_secs(TIMEOUT_SECS)).build() {
        Ok(client) => client,
        Err(_) => return TerminationOutcome::Failed { result: crate::generic_error(), can_retry: false },
    };
    let resp = match client.post(&req.url).header("X-Api-Key", &req.key).form(form).send().await {
        Ok(resp) => resp,
        // Keine Verbindung: der Server hat nichts erhalten.
        Err(e) if e.is_connect() => {
            return TerminationOutcome::Failed { result: crate::network_error(), can_retry: false };
        }
        // Gesendet, aber keine Antwort: kann ausgeführt worden sein.
        Err(_) => return TerminationOutcome::Failed { result: no_answer_warning(), can_retry: true },
    };
    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    interpret_termination_response(status, &body, info)
}

fn no_answer_warning() -> ActionResult {
    ActionResult {
        kind: ResultKind::Warning,
        message: "Es kam keine Antwort vom Server. Die Aktion wurde möglicherweise trotzdem ausgeführt.\n\n\
                  Ein erneuter Versuch ist unschädlich: Ist das Enddatum schon gesetzt, meldet LEGhub das und \
                  ändert nichts. Alternativ im Dashboard prüfen."
            .into(),
    }
}

/// Übersetzt Status und Body der Antwort in eine verständliche Rückmeldung.
pub fn interpret_termination_response(status: u16, body: &str, info: TerminationInfo) -> TerminationOutcome {
    let failed = |result| TerminationOutcome::Failed { result, can_retry: false };
    if status == 200 {
        let rows = if info.is_leg {
            serde_json::from_str::<LegResult>(body).ok().map(|r| {
                vec![ReportRow { measuring_point: None, ok: r.status == "OK", errors: r.errors }]
            })
        } else {
            serde_json::from_str::<MeasuringPointResponse>(body).ok().map(|r| {
                r.results
                    .into_iter()
                    .map(|r| ReportRow { measuring_point: r.measuring_point_number, ok: r.status == "OK", errors: r.errors })
                    .collect()
            })
        };
        return match rows {
            Some(rows) if !rows.is_empty() => TerminationOutcome::Report(TerminationReport { info, rows }),
            _ => TerminationOutcome::Failed {
                result: ActionResult {
                    kind: ResultKind::Warning,
                    message: "Die Antwort des Servers konnte nicht gelesen werden. Die Aktion wurde möglicherweise \
                              ausgeführt – bitte im Dashboard prüfen."
                        .into(),
                },
                can_retry: false,
            },
        };
    }
    if status == 403 {
        return failed(key_error());
    }
    let detail = serde_json::from_str::<ProblemBody>(body)
        .ok()
        .and_then(|p| p.detail)
        .map(|d| d.trim().to_string())
        .filter(|d| !d.is_empty());
    match status {
        400 => failed(error(format!(
            "Ungültige Eingabe{}",
            detail.map(|d| format!(": {d}")).unwrap_or_else(|| ". Bitte Gründungs-ID und Datum prüfen.".into())
        ))),
        404 => failed(error("Keine LEG mit dieser Gründungs-ID gefunden. Bitte die ID prüfen (interne LEG-ID wie im Dashboard).")),
        504 => TerminationOutcome::Failed { result: no_answer_warning(), can_retry: true },
        _ => failed(match detail {
            Some(d) => error(format!(
                "Die Aktion konnte nicht ausgeführt werden: {d}\n\nIn LEGhub und beim Netzbetreiber wurde nichts geändert."
            )),
            None => crate::generic_error(),
        }),
    }
}

// ----------------------------------------------------------------------------
// Texte
// ----------------------------------------------------------------------------

/// Herkunft eines Fehlers, für die Anzeige.
pub fn source_label(source: &str) -> &'static str {
    match source {
        "VNB" => "Netzbetreiber",
        "STORAGE" => "LEGhub-Datenbank",
        _ => "Prüfung in LEGhub",
    }
}

fn param(e: &MutationError, name: &str) -> String {
    e.params.get(name).cloned().flatten().unwrap_or_default()
}

fn date_param(e: &MutationError, name: &str) -> String {
    swiss_date(&param(e, name))
}

/// Deutscher Text zu einem Fehler. Unbekannte Codes und Fehler des Netzbetreibers zeigen den Server-Text
/// (beim Netzbetreiber ist der schon Deutsch oder Französisch).
pub fn german_message(e: &MutationError) -> String {
    let reason = || {
        let r = param(e, "reason");
        if r.is_empty() { String::new() } else { format!(" (Grund: {r})") }
    };
    match e.code.as_str() {
        "MEASURING_POINT_NOT_IN_LEG" => "Der Messpunkt gehört nicht zu dieser LEG.".into(),
        "MEASURING_POINT_ALREADY_DEACTIVATED" => format!(
            "Der Messpunkt ist bereits per {} ausgetreten. Ein gesetztes Enddatum kann nicht geändert werden.",
            date_param(e, "existingEndingDate")
        ),
        "ENDING_DATE_BEFORE_MEASURING_POINT_START" => format!(
            "Das Enddatum {} liegt vor dem Startdatum {} des Messpunkts {}.",
            date_param(e, "endingDate"),
            date_param(e, "startingDate"),
            param(e, "measuringPointNumber")
        ),
        "LEG_ALREADY_DISSOLVED" => format!(
            "Die LEG ist bereits per {} aufgelöst. Ein gesetztes Enddatum kann nicht geändert werden.",
            date_param(e, "existingEndingDate")
        ),
        "ENDING_DATE_BEFORE_LEG_START" => format!(
            "Das Enddatum {} liegt vor dem Startdatum {} der LEG.",
            date_param(e, "endingDate"),
            date_param(e, "startingDate")
        ),
        "CONFLICTING_ENDING_DATES" => "Derselbe Messpunkt wurde mit verschiedenen Enddaten angegeben.".into(),
        "ENDING_DATE_REQUIRED" | "MEASURING_POINT_FIELDS_REQUIRED" | "MEASURING_POINT_ENTRY_REQUIRED" => {
            "Messpunkt-Nummer oder Enddatum fehlt.".into()
        }
        "VNB_REPORT_FAILED" => "Die Meldung an den Netzbetreiber ist fehlgeschlagen (keine oder unlesbare Antwort). \
                                In LEGhub wurde nichts gespeichert. Ein erneuter Versuch ist unschädlich."
            .into(),
        "STORAGE_PERSIST_FAILED" => "Dem Netzbetreiber gemeldet, aber das Speichern in LEGhub ist fehlgeschlagen. \
                                     Bitte erneut versuchen – die erneute Meldung an den Netzbetreiber wird als bereits erledigt erkannt."
            .into(),
        "STORAGE_REJECTED" => format!(
            "Dem Netzbetreiber gemeldet, aber LEGhub hat das Speichern abgelehnt{}. Ein erneuter Versuch ändert \
             nichts – Netzbetreiber und LEGhub sind jetzt nicht mehr gleich. Bitte an die Entwicklung weitergeben.",
            reason()
        ),
        "STORAGE_ONLY_PERSIST_FAILED" => "Das Speichern in LEGhub ist fehlgeschlagen (Netzbetreiber wurde nicht \
                                          informiert). Es wurde nichts geändert – bitte erneut versuchen."
            .into(),
        "STORAGE_ONLY_REJECTED" => format!(
            "LEGhub hat das Speichern abgelehnt{} (Netzbetreiber wurde nicht informiert). Es wurde nichts geändert.",
            reason()
        ),
        _ if e.message.trim().is_empty() => format!("Unbekannter Fehler ({}).", e.code),
        _ => e.message.trim().to_string(),
    }
}

/// JJJJ-MM-TT → TT.MM.JJJJ; anderes bleibt unverändert.
pub fn swiss_date(iso: &str) -> String {
    match parse_iso_date(iso) {
        Some((y, m, d)) => format!("{d:02}.{m:02}.{y:04}"),
        None => iso.to_string(),
    }
}

/// Strikt JJJJ-MM-TT mit gültigem Kalendertag (der Server prüft genauso streng).
pub fn parse_iso_date(s: &str) -> Option<(i32, u32, u32)> {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let y: i32 = s[0..4].parse().ok()?;
    let m: u32 = s[5..7].parse().ok()?;
    let d: u32 = s[8..10].parse().ok()?;
    if !(1..=12).contains(&m) || d == 0 || d > days_in_month(y, m) {
        return None;
    }
    Some((y, m, d))
}

fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        2 if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Heutiges Datum (UTC) als JJJJ-MM-TT – für die Plausibilitäts-Hinweise genau genug.
fn today() -> String {
    let days = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() / 86_400).unwrap_or(0) as i64;
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Tage seit 1970-01-01 → Kalenderdatum (Algorithmus von Howard Hinnant).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

/// Plausibilitäts-Hinweise zum Enddatum. Gesperrt wird nichts – jedes Datum ist erlaubt –, aber ein
/// Tippfehler wäre unwiderruflich.
fn date_warnings(ending_date: &str, today: &str) -> Vec<String> {
    let mut warnings = Vec::new();
    let (Some((y, _, _)), Some((ty, _, _))) = (parse_iso_date(ending_date), parse_iso_date(today)) else {
        return warnings;
    };
    // Gleich lange ISO-Daten lassen sich als Text vergleichen.
    if ending_date < today {
        warnings.push(format!(
            "Das Enddatum {} liegt in der Vergangenheit. Der Netzbetreiber kann das ablehnen.",
            swiss_date(ending_date)
        ));
    }
    if y > ty + 1 {
        warnings.push(format!(
            "Das Enddatum {} liegt mehr als ein Jahr in der Zukunft. Bitte auf Tippfehler prüfen.",
            swiss_date(ending_date)
        ));
    }
    warnings
}

// ----------------------------------------------------------------------------
// Anzeige
// ----------------------------------------------------------------------------

/// Erklärung oberhalb des Formulars.
pub fn termination_guide_view(is_leg: bool) -> Element {
    rsx! {
        div { class: "guide",
            div { class: "guide-warn",
                strong { "Unwiderruflich. " }
                "Ein gesetztes Enddatum kann danach nicht mehr geändert oder gelöscht werden – weder hier noch im Dashboard. Es gibt keinen Probelauf; bitte Gründungs-ID, Datum und Messpunkte vor dem Absenden genau prüfen."
            }
            div { class: "guide-grid",
                div { class: "guide-section",
                    h3 { "Wann verwenden" }
                    ul {
                        if is_leg {
                            li { "Die LEG soll per Stichtag aufgelöst werden, und das Dashboard lässt das gewünschte Datum nicht zu." }
                        } else {
                            li { "Teilnehmende treten per Stichtag aus, und das Dashboard lässt das gewünschte Datum nicht zu." }
                        }
                        li { "Austritte/Auflösungen, die bisher per SQL am Stichtag gemacht wurden." }
                    }
                }
                div { class: "guide-section",
                    h3 { "Was passiert" }
                    ul {
                        if is_leg {
                            li { "Alle offenen Messpunkte der LEG und danach die LEG selbst werden dem Netzbetreiber (bzw. innosolv) gemeldet." }
                            li { "Danach setzt LEGhub das Enddatum der LEG, aller offenen Messpunkte und des Tarifs." }
                            li { "Messpunkte mit einem späteren Enddatum werden auf das Datum der LEG vorgezogen." }
                        } else {
                            li { "Jeder Messpunkt wird einzeln dem Netzbetreiber (bzw. innosolv) gemeldet und danach in LEGhub beendet." }
                            li { "Jeder Messpunkt hat ein eigenes Ergebnis – einzelne können klappen, andere nicht." }
                            li { "Noch nicht vom Netzbetreiber bestätigte Teilnahmen werden nur in LEGhub beendet." }
                        }
                        li { "Es ist derselbe Ablauf wie im Dashboard, inklusive allfälliger E-Mails." }
                    }
                }
                div { class: "guide-section",
                    h3 { "Enddatum" }
                    ul {
                        li { "Der letzte Tag, an dem die Teilnahme bzw. die LEG noch gilt (meist der Monatsletzte)." }
                        li { "Jedes Datum ist möglich. Abgelehnt wird nur ein Datum vor dem Start oder ein bereits gesetztes Enddatum – oder was der Netzbetreiber ablehnt." }
                    }
                }
                div { class: "guide-section",
                    h3 { "Ablauf" }
                    ol {
                        li { "Felder ausfüllen und «Ausführen»." }
                        li { "Im Popup Datum, LEG und Messpunkte prüfen und bestätigen." }
                        li { "Ergebnis prüfen; bei Bedarf «Erneut versuchen»." }
                        li { "Zusammenfassung ins Ticket kopieren." }
                    }
                }
            }
            div { class: "guide-section guide-force",
                h3 { "«Nur in LEGhub beenden» (Netzbetreiber nicht informieren)" }
                p {
                    "Für Netzbetreiber, deren Schnittstelle noch keine Austritte bzw. Auflösungen kann (z. B. Stadtwerk Winterthur). LEGhub setzt dann nur das eigene Enddatum. Der Netzbetreiber muss separat informiert werden – danach geht das auch über das Dashboard nicht mehr, weil das Enddatum bereits gesetzt ist."
                }
            }
        }
    }
}

/// Ergebnis eines Austritts bzw. einer Auflösung.
#[component]
pub fn TerminationReportView(report: TerminationReport) -> Element {
    let info = &report.info;
    let title = match (report.all_ok(), report.any_ok(), info.is_leg) {
        (true, _, true) => "LEG aufgelöst ✓".to_string(),
        (true, _, false) => "Austritt erledigt ✓".to_string(),
        (false, true, _) => "Teilweise erledigt – bitte die Fehler unten prüfen".to_string(),
        (false, false, true) => "LEG nicht aufgelöst".to_string(),
        (false, false, false) => "Kein Messpunkt ausgetreten".to_string(),
    };
    let reported = if info.storage_only {
        "Netzbetreiber NICHT informiert (nur in LEGhub) – bitte separat melden"
    } else {
        "mit Meldung an den Netzbetreiber"
    };
    rsx! {
        div { class: "summary",
            div { class: "summary-title", "{title}" }
            div { class: "summary-meta",
                "LEG {info.leg_id} · Enddatum {swiss_date(&info.ending_date)} · {reported}"
            }
            ul {
                for row in report.rows.iter() {
                    li {
                        if let Some(nr) = &row.measuring_point {
                            span { class: "mono", "{nr}" }
                            ": "
                        }
                        if row.ok {
                            "erledigt"
                        } else {
                            "nicht erledigt"
                            ul {
                                for e in row.errors.iter() {
                                    li {
                                        strong { "{source_label(&e.source)}: " }
                                        "{german_message(e)} "
                                        span { class: "mono", "({e.code})" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(is_leg: bool) -> TerminationInfo {
        TerminationInfo {
            is_leg,
            leg_id: "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".into(),
            ending_date: "2026-10-31".into(),
            measuring_points: vec![],
            storage_only: false,
        }
    }

    #[test]
    fn dates_are_parsed_strictly() {
        assert_eq!(parse_iso_date("2026-10-31"), Some((2026, 10, 31)));
        assert_eq!(parse_iso_date("2028-02-29"), Some((2028, 2, 29)));
        assert_eq!(parse_iso_date("2026-02-29"), None);
        assert_eq!(parse_iso_date("2026-02-30"), None);
        assert_eq!(parse_iso_date("31.10.2026"), None);
        assert_eq!(parse_iso_date("2026-1-5"), None);
        assert_eq!(swiss_date("2026-10-31"), "31.10.2026");
    }

    #[test]
    fn civil_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_741), (2026, 10, 15));
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
    }

    #[test]
    fn date_warnings_for_past_and_far_future() {
        assert!(date_warnings("2026-10-31", "2026-10-15").is_empty());
        assert!(date_warnings("2027-12-31", "2026-10-15").is_empty());
        assert_eq!(date_warnings("2026-09-30", "2026-10-15").len(), 1);
        assert_eq!(date_warnings("2062-10-31", "2026-10-15").len(), 1);
    }

    #[test]
    fn leg_result_ok() {
        let TerminationOutcome::Report(r) =
            interpret_termination_response(200, r#"{"status":"OK","retryable":false,"errors":[]}"#, info(true))
        else {
            panic!("expected report")
        };
        assert!(r.all_ok());
        assert!(r.kind() == ResultKind::Success);
        assert!(!r.any_retryable());
    }

    #[test]
    fn leg_result_error_is_translated() {
        let body = r#"{"status":"ERROR","retryable":false,"errors":[{"code":"LEG_ALREADY_DISSOLVED","source":"AJILA",
            "retryable":false,"message":"LEG is already dissolved as of 2026-06-30","params":{"existingEndingDate":"2026-06-30"}}]}"#;
        let TerminationOutcome::Report(r) = interpret_termination_response(200, body, info(true)) else { panic!() };
        assert!(r.kind() == ResultKind::Error);
        assert_eq!(
            german_message(&r.rows[0].errors[0]),
            "Die LEG ist bereits per 30.06.2026 aufgelöst. Ein gesetztes Enddatum kann nicht geändert werden."
        );
    }

    #[test]
    fn measuring_points_partially_ok_and_retryable() {
        let body = r#"{"results":[
            {"measuringPointNumber":"CH1","status":"OK","retryable":false,"errors":[]},
            {"measuringPointNumber":"CH2","status":"ERROR","retryable":true,"errors":[{"code":"STORAGE_PERSIST_FAILED","source":"STORAGE","retryable":true,"message":"x","params":{}}]}
        ]}"#;
        let TerminationOutcome::Report(r) = interpret_termination_response(200, body, info(false)) else { panic!() };
        assert!(r.kind() == ResultKind::Warning);
        assert!(r.any_retryable());
        assert!(!r.rows[0].retryable());
        assert!(r.rows[1].retryable());
    }

    #[test]
    fn storage_rejection_carries_the_reason_and_vnb_text_passes_through() {
        let e = MutationError {
            code: "STORAGE_ONLY_REJECTED".into(),
            source: "STORAGE".into(),
            retryable: false,
            message: String::new(),
            params: HashMap::from([("reason".to_string(), Some("Leg is dissolved".to_string()))]),
        };
        let text = german_message(&e);
        assert!(text.contains("Grund: Leg is dissolved"), "{text}");
        assert!(!text.contains("gemeldet,"), "{text}");
        let vnb = MutationError { code: "INVALID_ENDING_DATE".into(), source: "VNB".into(), message: "Ungültiges Enddatum".into(), ..e };
        assert_eq!(german_message(&vnb), "Ungültiges Enddatum");
    }

    #[test]
    fn error_statuses() {
        let failed = |status, body: &str| match interpret_termination_response(status, body, info(true)) {
            TerminationOutcome::Failed { result, can_retry } => (result, can_retry),
            TerminationOutcome::Report(_) => panic!("expected failure"),
        };
        assert_eq!(failed(403, "").0.message, key_error().message);
        assert!(failed(400, r#"{"status":400,"detail":"endingDate must be a valid day"}"#).0.message.contains("endingDate"));
        assert!(failed(404, r#"{"status":404,"detail":"No LEG"}"#).0.message.contains("Keine LEG"));
        let (timeout, can_retry) = failed(504, "");
        assert!(timeout.kind == ResultKind::Warning && can_retry);
        assert!(failed(500, r#"{"status":500,"detail":"No tenant configuration"}"#).0.message.contains("No tenant"));
        // Unlesbare 200-Antwort: möglicherweise ausgeführt, also Warnung statt Fehler.
        assert!(failed(200, "<html>").0.kind == ResultKind::Warning);
    }

    #[test]
    fn ticket_text_names_storage_only() {
        let mut i = info(false);
        i.storage_only = true;
        let r = TerminationReport {
            info: i,
            rows: vec![ReportRow { measuring_point: Some("CH1".into()), ok: true, errors: vec![] }],
        };
        let text = r.to_ticket_text(Env::Test);
        assert!(text.contains("Netzbetreiber informiert: nein"), "{text}");
        assert!(text.contains("Enddatum: 31.10.2026"), "{text}");
        assert!(text.contains("Messpunkt CH1: erledigt"), "{text}");
    }
}
