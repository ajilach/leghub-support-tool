//! Messpunkte: «Messpunktnummer ändern».
//!
//! Korrigiert die Nummer des offenen Messpunkts einer LEG – nur in LEGhub, der Netzbetreiber wird
//! nicht informiert. Ausgetretene Messpunkte bleiben als Historie unverändert.
//!
//! Server-Seite: leg-Servlet `supportChangeMeasuringPointNumber`. Fehler kommen als
//! `{status, detail}` mit englischem Text; die bekannten Fälle werden hier auf Deutsch übersetzt.

use std::time::Duration;

use serde::Deserialize;

use crate::{error, generic_error, key_error, network_error, ActionResult, PendingRequest, ProblemBody, RequestBody, ResultKind};

/// Letzter Teil der Server-Adresse dieser Aktion.
pub const CHANGE_NUMBER_PATH: &str = "supportChangeMeasuringPointNumber";
const CURRENT_NUMBER: &str = "measuringPointNumber";
const NEW_NUMBER: &str = "newMeasuringPointNumber";
/// Eine reine Datenbank-Änderung – länger als eine Minute heisst: Antwort verloren, nicht «noch am Arbeiten».
const TIMEOUT_SECS: u64 = 60;

/// Messpunktnummer ändern?
pub fn is_change_number_action(path_suffix: &str) -> bool {
    path_suffix == CHANGE_NUMBER_PATH
}

/// Schweizer Messpunktnummer: «CH» und 31 Ziffern, Grossbuchstaben oder Bindestriche.
fn is_valid_measuring_point_number(number: &str) -> bool {
    number.len() == 33
        && number.starts_with("CH")
        && number[2..].chars().all(|c| c.is_ascii_digit() || c.is_ascii_uppercase() || c == '-')
}

/// Prüft beide Nummern vor dem Senden und schreibt sie in Grossbuchstaben ins Formular zurück. Die
/// bisherige Nummer muss nur aus erlaubten Zeichen bestehen – auch eine falsch erfasste soll korrigierbar sein.
pub fn normalize_change_number(form: &mut [(String, String)]) -> Result<(), &'static str> {
    for (key, value) in form.iter_mut() {
        if key == CURRENT_NUMBER {
            *value = value.trim().to_uppercase();
            if value.is_empty() || !value.chars().all(|c| c.is_ascii_digit() || c.is_ascii_uppercase() || c == '-') {
                return Err("Die bisherige Messpunktnummer darf nur Ziffern, Buchstaben und Bindestriche enthalten.");
            }
        } else if key == NEW_NUMBER {
            *value = value.trim().to_uppercase();
            if !is_valid_measuring_point_number(value) {
                return Err("Die neue Messpunktnummer beginnt mit «CH», gefolgt von 31 Ziffern, Buchstaben oder Bindestrichen \
                            (33 Zeichen), z. B. «CH1015001234500010060400007001001».");
            }
        }
    }
    let get = |name: &str| form.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str());
    if get(CURRENT_NUMBER) == get(NEW_NUMBER) {
        return Err("Die neue Messpunktnummer ist gleich der bisherigen.");
    }
    Ok(())
}

/// Sendet die Änderung und wertet die Antwort aus.
pub async fn send_change_number_request(req: &PendingRequest) -> ActionResult {
    let RequestBody::Form(form) = &req.body else {
        return generic_error();
    };
    let client = reqwest::Client::new();
    let sent = client
        .post(&req.url)
        .header("X-Api-Key", &req.key)
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .form(form)
        .send()
        .await;
    match sent {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            interpret_change_number_response(status, &body)
        }
        Err(e) if e.is_timeout() => ActionResult {
            kind: ResultKind::Warning,
            message: "Der Server hat nicht rechtzeitig geantwortet. Die Nummer wurde möglicherweise trotzdem geändert. \
                      Bitte im Dashboard prüfen, bevor Sie es erneut versuchen."
                .into(),
        },
        Err(_) => network_error(),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChangedNumber {
    measuring_point_number: String,
    new_measuring_point_number: String,
}

/// Übersetzt die Antwort des Servers in eine verständliche Rückmeldung.
pub fn interpret_change_number_response(status: u16, body: &str) -> ActionResult {
    if (200..300).contains(&status) {
        let message = match serde_json::from_str::<ChangedNumber>(body) {
            Ok(changed) => format!(
                "Erledigt ✓  Die Messpunktnummer wurde von {} auf {} geändert.",
                changed.measuring_point_number, changed.new_measuring_point_number
            ),
            Err(_) => "Erledigt ✓  Die Messpunktnummer wurde geändert.".into(),
        };
        return ActionResult { kind: ResultKind::Success, message };
    }
    let detail = serde_json::from_str::<ProblemBody>(body)
        .ok()
        .and_then(|p| p.detail)
        .map(|d| d.trim().to_string())
        .filter(|d| !d.is_empty());
    match status {
        403 => key_error(),
        404 => error(
            "Kein offener Messpunkt mit dieser Nummer in dieser LEG gefunden. Bitte Gründungs-ID und bisherige \
             Messpunktnummer prüfen – ausgetretene Messpunkte können nicht geändert werden.",
        ),
        409 => error(
            "Die neue Messpunktnummer ist bereits einem aktiven Messpunkt zugeordnet (überschneidender Zeitraum). \
             Es wurde nichts geändert.",
        ),
        400 => error(match detail.as_deref() {
            Some(d) if d.contains("legId") => "Bitte eine gültige Gründungs-ID angeben.".to_string(),
            Some(d) if d.contains("must differ") || d.contains("equals the current") => {
                "Die neue Messpunktnummer ist gleich der bisherigen.".to_string()
            }
            Some(d) => format!("Ungültige Eingabe: {d}"),
            None => "Ungültige Eingabe. Bitte die Felder prüfen.".to_string(),
        }),
        _ => match detail {
            Some(d) => error(format!("Die Aktion konnte nicht ausgeführt werden: {d}")),
            None => generic_error(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OLD: &str = "CH1015001234500010060400007001001";
    const NEW: &str = "CH1015001234500010060400007001002";

    fn form(old: &str, new: &str) -> Vec<(String, String)> {
        vec![
            ("legId".into(), "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".into()),
            (CURRENT_NUMBER.into(), old.into()),
            (NEW_NUMBER.into(), new.into()),
        ]
    }

    #[test]
    fn normalize_upper_cases_and_trims_both_numbers() {
        let mut f = form(&format!(" {} ", OLD.to_lowercase()), &NEW.to_lowercase());
        assert!(normalize_change_number(&mut f).is_ok());
        assert_eq!(f[1].1, OLD);
        assert_eq!(f[2].1, NEW);
        assert_eq!(f[0].1, "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee");
    }

    #[test]
    fn normalize_accepts_a_wrongly_formatted_current_number() {
        let mut f = form("ch123", NEW);
        assert!(normalize_change_number(&mut f).is_ok());
        assert_eq!(f[1].1, "CH123");
    }

    #[test]
    fn normalize_rejects_invalid_numbers() {
        assert!(normalize_change_number(&mut form("CH 123", NEW)).is_err());
        assert!(normalize_change_number(&mut form("CH123/..", NEW)).is_err());
        assert!(normalize_change_number(&mut form(OLD, "CH123")).is_err());
        assert!(normalize_change_number(&mut form(OLD, "DE1015001234500010060400007001002")).is_err());
        assert!(normalize_change_number(&mut form(OLD, "CH1015001234500010060400007001002X")).is_err());
        assert!(normalize_change_number(&mut form(OLD, "CH10150012345000100604000070010_2")).is_err());
    }

    #[test]
    fn normalize_rejects_identical_numbers() {
        assert!(normalize_change_number(&mut form(OLD, &OLD.to_lowercase())).is_err());
    }

    #[test]
    fn success_names_both_numbers() {
        let body = format!(r#"{{"measuringPointNumber":"{OLD}","newMeasuringPointNumber":"{NEW}"}}"#);
        let result = interpret_change_number_response(200, &body);
        assert!(result.kind == ResultKind::Success);
        assert!(result.message.contains(OLD) && result.message.contains(NEW), "{}", result.message);
    }

    #[test]
    fn known_errors_are_translated() {
        assert!(interpret_change_number_response(404, r#"{"status":404,"detail":"No open measuring point"}"#)
            .message
            .contains("Kein offener Messpunkt"));
        assert!(interpret_change_number_response(409, r#"{"status":409,"detail":"already exists"}"#)
            .message
            .contains("bereits einem aktiven Messpunkt"));
        assert!(interpret_change_number_response(400, r#"{"status":400,"detail":"legId must be a UUID"}"#)
            .message
            .contains("Gründungs-ID"));
        assert!(interpret_change_number_response(403, "").message.contains("Zugangsschlüssel"));
    }

    #[test]
    fn other_errors_show_detail_or_fallback() {
        let with_detail = interpret_change_number_response(502, r#"{"status":502,"detail":"Storage failed"}"#);
        assert!(with_detail.kind == ResultKind::Error && with_detail.message.contains("Storage failed"));
        assert!(interpret_change_number_response(500, "").message == generic_error().message);
    }
}
