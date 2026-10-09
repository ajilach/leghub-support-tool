//! LegHub Support-Tool — kleine Desktop-App, um LegHub-Support-Aktionen auszulösen.
//!
//! Die Oberfläche ist bewusst ohne technische Begriffe gehalten: links wählt man
//! eine Aktion, füllt ein bis zwei Felder aus und drückt "Ausführen". In den
//! Einstellungen wählt man zwischen Test- und Produktiv-Umgebung und hinterlegt
//! den Zugangsschlüssel.

use std::collections::HashMap;

use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

mod flows;

use flows::Overview;

const STYLE: Asset = asset!("/assets/style.css");
const LOGO: Asset = asset!("/assets/leghub_logo.svg");

// ----------------------------------------------------------------------------
// Aktions-Modell (fest im Code hinterlegt)
// ----------------------------------------------------------------------------

struct Field {
    /// Technischer Parametername, wird an den Server gesendet (nie angezeigt).
    key: &'static str,
    /// Beschriftung für die Nutzerin / den Nutzer.
    label: &'static str,
    /// Kurzer Hinweis, wo diese Angabe zu finden ist (leer = kein Hinweis).
    hint: &'static str,
    required: bool,
}

struct Action {
    group: &'static str,
    title: &'static str,
    description: &'static str,
    /// Letzter Teil der Adresse hinter /bin/com/ajila/forms/industry/leg/
    path_suffix: &'static str,
    fields: &'static [Field],
}

static ACTIONS: &[Action] = &[
    Action {
        group: "Gründungen",
        title: "Gründung neu starten",
        description: "Startet den gesamten Gründungs-Ablauf mit den bereits erfassten Daten noch einmal von vorne.",
        path_suffix: "restartFounding",
        fields: &[Field { key: "legId", label: "Gründungs-ID", hint: "ID wie im Dashboard aufgelistet", required: true }],
    },
    Action {
        group: "Gründungen",
        title: "Unterschrifts-Anfrage erneut senden",
        description: "Sendet die Einladung zur Unterschrift für eine Gründung noch einmal.",
        path_suffix: "retriggerFoundingSignature",
        fields: &[Field { key: "legId", label: "Gründungs-ID", hint: "ID wie im Dashboard aufgelistet", required: true }],
    },
    Action {
        group: "Gründungen",
        title: "Gründung durch Netzbetreiber annehmen",
        description: "Nimmt die Gründung im Namen des Netzbetreibers an.",
        path_suffix: "founding/acceptVnb",
        fields: &[Field { key: "id", label: "Gründungs-ID", hint: "ID wie im Dashboard aufgelistet", required: true }],
    },
    Action {
        group: "Gründungen",
        title: "Gründung durch Netzbetreiber ablehnen",
        description: "Lehnt die Gründung im Namen des Netzbetreibers ab.",
        path_suffix: "founding/rejectVnb",
        fields: &[
            Field { key: "id", label: "Gründungs-ID", hint: "ID wie im Dashboard aufgelistet", required: true },
            Field { key: "message", label: "Begründung", hint: "Grund der Ablehnung (wird der Person mitgeteilt)", required: false },
        ],
    },
    Action {
        group: "Gründungen",
        title: "Persistierung der Gründung erneut versuchen",
        description: "Versucht nur das Speichern der Gründung erneut, das zuvor fehlgeschlagen ist. Der übrige Ablauf wird nicht wiederholt.",
        path_suffix: "founding/retryVnb",
        fields: &[Field { key: "id", label: "Gründungs-ID", hint: "ID wie im Dashboard aufgelistet", required: true }],
    },
    Action {
        group: "Gründungen",
        title: "Gründung anpassen",
        description: "Ändert einzelne Angaben einer bestehenden Gründung. Nur die angegebenen Felder werden überschrieben, alles andere bleibt unverändert.",
        path_suffix: "patchFounding",
        fields: &[
            Field { key: "legId", label: "Gründungs-ID", hint: "ID wie im Dashboard aufgelistet", required: true },
        ],
    },
    Action {
        group: "Teilnahmen",
        title: "Teilnahme neu starten",
        description: "Startet den gesamten Teilnahme-Ablauf mit den bereits erfassten Daten noch einmal von vorne.",
        path_suffix: "restartParticipation",
        fields: &[Field { key: "participationId", label: "Teilnahme-ID", hint: "ID wie im Dashboard aufgelistet", required: true }],
    },
    Action {
        group: "Teilnahmen",
        title: "Unterschrifts-Anfrage erneut senden",
        description: "Sendet die Einladung zur Unterschrift für eine Teilnahme noch einmal.",
        path_suffix: "retriggerParticipationSignature",
        fields: &[Field { key: "participationId", label: "Teilnahme-ID", hint: "ID wie im Dashboard aufgelistet", required: true }],
    },
    Action {
        group: "Teilnahmen",
        title: "Teilnahme durch Netzbetreiber annehmen",
        description: "Nimmt die Teilnahme im Namen des Netzbetreibers an.",
        path_suffix: "participation/acceptVnb",
        fields: &[Field { key: "id", label: "Teilnahme-ID", hint: "ID wie im Dashboard aufgelistet", required: true }],
    },
    Action {
        group: "Teilnahmen",
        title: "Teilnahme durch Netzbetreiber ablehnen",
        description: "Lehnt die Teilnahme im Namen des Netzbetreibers ab.",
        path_suffix: "participation/rejectVnb",
        fields: &[
            Field { key: "id", label: "Teilnahme-ID", hint: "ID wie im Dashboard aufgelistet", required: true },
            Field { key: "message", label: "Begründung", hint: "Grund der Ablehnung (wird der Person mitgeteilt)", required: false },
        ],
    },
    Action {
        group: "Teilnahmen",
        title: "Persistierung der Teilnahme erneut versuchen",
        description: "Versucht nur das Speichern der Teilnahme erneut, das zuvor fehlgeschlagen ist. Der übrige Ablauf wird nicht wiederholt.",
        path_suffix: "participation/retryVnb",
        fields: &[Field { key: "id", label: "Teilnahme-ID", hint: "ID wie im Dashboard aufgelistet", required: true }],
    },
    Action {
        group: "Teilnahmen",
        title: "Teilnahme durch Gründer annehmen",
        description: "Nimmt die Teilnahme im Namen des Gründers an.",
        path_suffix: "participation/acceptFounder",
        fields: &[Field { key: "id", label: "Teilnahme-ID", hint: "ID wie im Dashboard aufgelistet", required: true }],
    },
    Action {
        group: "Teilnahmen",
        title: "Teilnahme durch Gründer ablehnen",
        description: "Lehnt die Teilnahme im Namen des Gründers ab.",
        path_suffix: "participation/rejectFounder",
        fields: &[
            Field { key: "id", label: "Teilnahme-ID", hint: "ID wie im Dashboard aufgelistet", required: true },
            Field { key: "message", label: "Begründung", hint: "Grund der Ablehnung (wird der Person mitgeteilt)", required: false },
        ],
    },
    Action {
        group: "Teilnahmen",
        title: "Teilnahme anpassen",
        description: "Ändert einzelne Angaben einer bestehenden Teilnahme. Nur die angegebenen Felder werden überschrieben, alles andere bleibt unverändert.",
        path_suffix: "patchParticipation",
        fields: &[
            Field { key: "legId", label: "Gründungs-ID", hint: "ID der zugehörigen Gründung", required: true },
            Field { key: "participationId", label: "Teilnahme-ID", hint: "ID wie im Dashboard aufgelistet", required: true },
        ],
    },
    Action {
        group: "Unterschriften",
        title: "Unterschrift als abgeschlossen markieren",
        description: "Behandelt eine Unterschrift so, als wäre sie fertig unterzeichnet worden.",
        path_suffix: "signatureProcessCompleted",
        fields: &[Field { key: "documentId", label: "Dokument-ID", hint: "ID wie im SC Sign Mail angezeigt", required: true }],
    },
    Action {
        group: "Löschen",
        title: "LEG löschen",
        description: "Löscht eine LEG samt allen Teilnahmen, Messpunkten und Verträgen. Zuerst wird immer ein Probelauf gemacht, der zeigt, was gelöscht würde.",
        path_suffix: "deleteLeg",
        fields: &[
            Field { key: "legId", label: "Gründungs-ID", hint: "Interne LEG-ID wie im Dashboard (im Link «leg-detail?legId=…»)", required: true },
            Field { key: "ticket", label: "Ticket", hint: "Support-Ticket, dem die Löschung zugeordnet wird, z. B. LHSD-1091", required: true },
            Field { key: "reason", label: "Begründung", hint: "Kurzer Grund fürs Audit, z. B. «Duplikat» oder «Test-LEG». Keine Personendaten (Namen, E-Mail) – der Text landet in Server-Logs.", required: false },
        ],
    },
    Action {
        group: "Löschen",
        title: "Teilnahme löschen",
        description: "Löscht eine ganze Teilnahme samt Messpunkten und Verträgen. Messpunkte, die noch zu einer anderen Teilnahme gehören, bleiben bestehen – nur die Verknüpfung wird entfernt. Zuerst wird immer ein Probelauf gemacht.",
        path_suffix: "deleteParticipation",
        fields: &[
            Field { key: "legId", label: "Gründungs-ID", hint: "ID der zugehörigen Gründung", required: true },
            Field { key: "participantId", label: "Teilnehmer-ID", hint: "Teilnehmer-ID, wie sie im Ticket bzw. Dashboard steht", required: true },
            Field { key: "ticket", label: "Ticket", hint: "Support-Ticket, dem die Löschung zugeordnet wird, z. B. LHSD-1091", required: true },
            Field { key: "reason", label: "Begründung", hint: "Kurzer Grund fürs Audit, z. B. «Duplikat» oder «Test-LEG». Keine Personendaten (Namen, E-Mail) – der Text landet in Server-Logs.", required: false },
        ],
    },
    Action {
        group: "Löschen",
        title: "Messpunkte löschen",
        description: "Löscht nur die angegebenen Messpunkte einer Teilnahme. Die Teilnahme selbst bleibt bestehen. Zuerst wird immer ein Probelauf gemacht.",
        path_suffix: "deleteParticipation",
        fields: &[
            Field { key: "legId", label: "Gründungs-ID", hint: "ID der zugehörigen Gründung", required: true },
            Field { key: "participantId", label: "Teilnehmer-ID", hint: "Teilnehmer-ID, wie sie im Ticket bzw. Dashboard steht", required: true },
            Field { key: "measuringPointNumber", label: "Messpunkte", hint: "Nur diese Messpunkte der Teilnahme werden gelöscht. Mehrere Nummern durch Komma oder Zeilenumbruch trennen, z. B. «CH1015001234500010060400007001001, CH1015001234500010060400007001002». Gross-/Kleinschreibung spielt keine Rolle.", required: true },
            Field { key: "ticket", label: "Ticket", hint: "Support-Ticket, dem die Löschung zugeordnet wird, z. B. LHSD-1091", required: true },
            Field { key: "reason", label: "Begründung", hint: "Kurzer Grund fürs Audit, z. B. «Duplikat» oder «Test-LEG». Keine Personendaten (Namen, E-Mail) – der Text landet in Server-Logs.", required: false },
        ],
    },
];

/// Reihenfolge der Gruppen im Seitenpanel.
static GROUPS: &[&str] = &["Gründungen", "Teilnahmen", "Unterschriften", "Löschen"];

// ----------------------------------------------------------------------------
// Lösch-Aktionen (immer zuerst als Probelauf)
// ----------------------------------------------------------------------------

/// Lösch-Aktionen laufen zuerst als Probelauf und werden erst danach wirklich ausgeführt.
fn is_delete_action(path_suffix: &str) -> bool {
    matches!(path_suffix, "deleteLeg" | "deleteParticipation")
}

/// Felder, in die mehrere Werte eingetragen werden können (je Wert ein eigener Parameter).
fn is_multi_value(key: &str) -> bool {
    key == "measuringPointNumber"
}

/// Teilt eine Eingabe mit mehreren Werten an Komma, Semikolon, Leerzeichen und Zeilenumbruch auf.
fn split_multi_value(raw: &str) -> Vec<String> {
    raw.split(|c: char| c == ',' || c == ';' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// Ticket-Nummer im Format `^[A-Z][A-Z0-9]*-\d+$`, z. B. LHSD-1091.
fn is_valid_ticket(ticket: &str) -> bool {
    let Some((project, number)) = ticket.split_once('-') else { return false };
    let mut chars = project.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_uppercase())
        && chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        && !number.is_empty()
        && number.chars().all(|c| c.is_ascii_digit())
}

/// Ausführliche Erklärung einer Lösch-Aktion (Quelle: Confluence «Support-Endpoints: LEG,
/// Teilnahme und Messpunkte löschen»). Löschen ist heikler als die übrigen Aktionen, daher
/// mehr Text als dort.
struct DeleteGuide {
    /// Typische Fälle, für die diese Aktion gedacht ist.
    when: &'static [&'static str],
    deletes: &'static [&'static str],
    keeps: &'static [&'static str],
    /// Zusätzliche Hinweise (leer = keine).
    notes: &'static [&'static str],
    /// Wann «Schutz übersteuern» nötig und richtig ist.
    force_when: &'static [&'static str],
    /// Was auch mit «Schutz übersteuern» nicht geht.
    force_never: &'static str,
}

const FORCE_NEVER_OTHER_LEG: &str =
    "Gehört ein Messpunkt auch zu einer anderen LEG, ist das Löschen auch mit «Schutz übersteuern» nicht möglich und muss manuell mit der Entwicklung geklärt werden.";

static GUIDE_LEG: DeleteGuide = DeleteGuide {
    when: &[
        "Test- oder Vorführ-LEGs",
        "Duplikate oder falsch erfasste LEGs",
        "Abgebrochene oder vom Netzbetreiber abgelehnte Gründungen",
    ],
    deletes: &[
        "die LEG mit Energiemodell und Preismodellen",
        "alle Teilnahmen und Messpunkte der LEG",
        "Verträge, die nur an dieser LEG hängen",
    ],
    keeps: &[
        "Benutzer (auch der Gründer), Adressen und Netzadressen",
        "Verträge, die auch zu einer anderen LEG gehören – nur deren Verknüpfung zu dieser LEG wird entfernt",
    ],
    notes: &["Alles geschieht in einem Schritt: schlägt etwas fehl, wird nichts gelöscht."],
    force_when: &[
        "die LEG hat den Status «Wartet auf Produzent», «Wartet auf Verbraucher», «Aktiv» oder «Beendet», ist also vom Netzbetreiber akzeptiert,",
        "die LEG hat bereits eine externe ID vom Netzbetreiber,",
        "die LEG ist aufgelöst (Enddatum gesetzt) oder",
        "eine Teilnahme ist «Aktiv» oder «Beendet».",
    ],
    force_never: "Gehört ein Messpunkt oder ein Vertrag auch zu einer anderen LEG, ist das Löschen auch mit «Schutz übersteuern» nicht möglich und muss manuell mit der Entwicklung geklärt werden.",
};

static GUIDE_PARTICIPATION: DeleteGuide = DeleteGuide {
    when: &[
        "Hängender oder falsch erfasster Beitritt – der Kunde soll sich neu anmelden",
        "Beitritt stornieren",
    ],
    deletes: &[
        "alle Messpunkte dieser Teilnahme in dieser LEG samt den Teilnahme-Einträgen",
        "Verträge, die nur an dieser LEG hängen – sofern der Unterzeichner danach keinen Messpunkt mehr in der LEG hat",
    ],
    keeps: &[
        "die LEG, die Benutzer und die Teilnahmen anderer Teilnehmer",
        "dieselbe Teilnehmer-ID in einer anderen LEG",
        "Messpunkte, an denen noch eine andere Teilnahme derselben LEG hängt – nur die Verknüpfung dieser Teilnahme wird entfernt",
    ],
    notes: &[
        "Danach kann sich der Kunde mit demselben Messpunkt neu anmelden. Bei privaten LEGs muss die Vertretung den Beitrittslink aus der Gründungsbestätigung erneut teilen.",
    ],
    force_when: &[
        "ein Messpunkt gehört dem Gründer der LEG – dann ist meist «LEG löschen» der richtige Weg, oder",
        "ein Messpunkt ist «Aktiv» oder «Beendet», der Netzbetreiber kennt ihn also bereits.",
    ],
    force_never: FORCE_NEVER_OTHER_LEG,
};

static GUIDE_MEASURING_POINTS: DeleteGuide = DeleteGuide {
    when: &["Einzelne, falsch erfasste Messpunkte eines Teilnehmers (meist zusammen mit «Schutz übersteuern»)"],
    deletes: &[
        "nur die angegebenen Messpunkte dieser Teilnahme – hat ein Prosumer dieselbe Nummer für Bezug und Einspeisung, werden beide gelöscht",
        "Verträge, die nur an dieser LEG hängen – sofern der Unterzeichner danach keinen Messpunkt mehr in der LEG hat",
    ],
    keeps: &[
        "die Teilnahme mit ihren übrigen Messpunkten",
        "die LEG, die Benutzer und die Teilnahmen anderer Teilnehmer",
    ],
    notes: &[
        "Gehört eine angegebene Nummer nicht zu dieser Teilnahme, wird gar nichts gelöscht.",
        "Werden alle Messpunkte der Teilnahme angegeben, ist das dasselbe wie «Teilnahme löschen».",
        "Die Schutzregeln prüfen nur die angegebenen Messpunkte.",
    ],
    force_when: &[
        "ein angegebener Messpunkt gehört dem Gründer der LEG oder",
        "ein angegebener Messpunkt ist «Aktiv» oder «Beendet», der Netzbetreiber kennt ihn also bereits.",
    ],
    force_never: FORCE_NEVER_OTHER_LEG,
};

/// Erklärung zur Lösch-Aktion mit diesem Titel (zwei Aktionen teilen sich `deleteParticipation`).
fn delete_guide(action: &Action) -> Option<&'static DeleteGuide> {
    match (action.path_suffix, action.title) {
        ("deleteLeg", _) => Some(&GUIDE_LEG),
        ("deleteParticipation", "Messpunkte löschen") => Some(&GUIDE_MEASURING_POINTS),
        ("deleteParticipation", _) => Some(&GUIDE_PARTICIPATION),
        _ => None,
    }
}

/// Zusammenfassung, die der Server bei einer Löschung (oder einem Probelauf) liefert.
#[derive(Clone, PartialEq, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeletionSummary {
    /// Bewusst ohne Default: fehlt die Angabe, gilt die Antwort als unlesbar.
    dry_run: bool,
    #[serde(default)]
    forced: bool,
    #[serde(default)]
    ticket: String,
    #[serde(default)]
    leg_internal_id: String,
    #[serde(default)]
    leg_name: Option<String>,
    #[serde(default)]
    leg_deleted: bool,
    #[serde(default)]
    participant_ids: Vec<String>,
    #[serde(default)]
    measuring_point_numbers: Vec<String>,
    #[serde(default)]
    detached_measuring_point_numbers: Vec<String>,
    #[serde(default)]
    measuring_points: u64,
    #[serde(default)]
    contracts: u64,
}

impl DeletionSummary {
    /// LEG-Name mit ID, oder nur die ID, falls kein Name bekannt ist.
    fn leg_label(&self) -> String {
        match self.leg_name.as_deref().map(str::trim) {
            Some(name) if !name.is_empty() => format!("«{name}» ({})", self.leg_internal_id),
            _ => self.leg_internal_id.clone(),
        }
    }

    /// Einzeilige Kurzfassung, z. B. für das Bestätigungs-Popup.
    fn short(&self) -> String {
        let mut parts = Vec::new();
        if self.leg_deleted {
            parts.push(format!("LEG {}", self.leg_label()));
        }
        if !self.participant_ids.is_empty() {
            parts.push(format!("{} Teilnahme(n)", self.participant_ids.len()));
        }
        parts.push(format!("{} Messpunkt(e)", self.measuring_points));
        parts.push(format!("{} Vertrag/Verträge", self.contracts));
        parts.join(", ")
    }

    /// Klartext zum Kopieren ins Ticket.
    fn to_ticket_text(&self, env: Env) -> String {
        let mut lines = vec![
            format!("LEGhub Support-Tool: Löschung ausgeführt ({})", env.label()),
            format!("Ticket: {}", self.ticket),
            format!("LEG: {}", self.leg_label()),
            format!("LEG gelöscht: {}", if self.leg_deleted { "ja" } else { "nein" }),
        ];
        if !self.participant_ids.is_empty() {
            lines.push(format!("Teilnahmen: {}", self.participant_ids.join(", ")));
        }
        lines.push(format!("Messpunkte ({}): {}", self.measuring_points, self.measuring_point_numbers.join(", ")));
        lines.push(format!("Verträge: {}", self.contracts));
        if !self.detached_measuring_point_numbers.is_empty() {
            lines.push(format!(
                "Nur Verknüpfung entfernt (Messpunkte bleiben bestehen): {}",
                self.detached_measuring_point_numbers.join(", ")
            ));
        }
        if self.forced {
            lines.push("Schutz übersteuert (force): ja".into());
        }
        lines.join("\n")
    }
}

/// Fehlerantwort des Servers (ProblemDetail von Storage oder `{status, detail}` von leg).
#[derive(Deserialize)]
struct ProblemBody {
    detail: Option<String>,
}

/// Ergebnis eines Lösch-Aufrufs.
enum DeleteOutcome {
    Summary(DeletionSummary),
    Failed {
        result: ActionResult,
        /// Eine Schutzregel greift, die sich mit «Schutz übersteuern» umgehen lässt.
        can_force: bool,
    },
}

/// Erfolgreicher Probelauf: genau dieser Request darf danach wirklich ausgeführt werden.
#[derive(Clone, PartialEq)]
struct DryRun {
    request: PendingRequest,
    summary: DeletionSummary,
}

// ----------------------------------------------------------------------------
// Änderbare Felder für die "anpassen"-Aktionen (Feld-Auswahl statt JSON)
// ----------------------------------------------------------------------------

/// Art des Werts eines änderbaren Felds — bestimmt das Eingabeelement und die
/// JSON-Umwandlung.
#[derive(Clone, Copy, PartialEq)]
enum ValueKind {
    Text,
    Number,
    /// Ja / Nein.
    Bool,
    /// Datum, wird als Zeitpunkt (…T00:00:00Z) gesendet.
    Date,
    /// Datum ohne Uhrzeit (JJJJ-MM-TT), z. B. Geburtsdatum.
    LocalDate,
    /// Auswahl aus festen Werten: (technischer Wert, Beschriftung).
    Choice(&'static [(&'static str, &'static str)]),
}

struct PatchField {
    /// Punkt-Pfad in der gespeicherten Struktur, z. B. "energyConfiguration.discountLevel".
    path: &'static str,
    label: &'static str,
    kind: ValueKind,
}

// --- Auswahllisten ---
const VISIBILITY: &[(&str, &str)] = &[("PUBLIC", "Öffentlich"), ("PRIVATE", "Privat")];
const DISCOUNT: &[(&str, &str)] = &[("HIGH", "Hoch"), ("LOW", "Tief")];
const GRID_LEVEL: &[(&str, &str)] = &[
    ("SEVEN", "Netzebene 7"),
    ("FIVE", "Netzebene 5"),
    ("FIVE_EXCLUSIVE", "Netzebene 5 (exklusiv)"),
];
const BILLING_TYPE: &[(&str, &str)] = &[
    ("MANAGED", "Verwaltet"),
    ("DIY", "Selbst"),
    ("SHARED", "Geteilt"),
];
const PRICING_MODEL: &[(&str, &str)] = &[
    ("STANDARD_TARIFF", "Standardtarif"),
    ("INDIVIDUAL_FIXED_PRICE", "Individueller Fixpreis"),
    ("CUSTOMIZED_STANDARD_TARIFF", "Angepasster Standardtarif"),
    ("CUSTOMIZED_PRIZING_MODEL", "Angepasstes Preismodell"),
];
const PARTICIPANT_TYPE: &[(&str, &str)] = &[
    ("CONSUMER", "Verbraucher"),
    ("PRODUCER", "Produzent"),
    ("PROSUMER", "Prosumer"),
    ("NEITHER", "Keiner"),
];
const LEGAL_FORM: &[(&str, &str)] = &[
    ("PRIVATE_PERSON", "Privatperson"),
    ("ENTERPRISE", "Unternehmen"),
    ("LOCAL_AUTHORITY", "Gemeinde / Behörde"),
    ("ASSOCIATION", "Verein"),
    ("COOPERATIVE", "Genossenschaft"),
    ("FOUNDATION", "Stiftung"),
    ("OTHER", "Andere"),
];
const LEG_STATE: &[(&str, &str)] = &[
    ("DRAFT", "Entwurf"),
    ("WAITING_FOR_VNB_APPROVAL", "Wartet auf Netzbetreiber"),
    ("WAITING_FOR_SIGNATURE", "Wartet auf Unterschrift"),
    ("WAITING_FOR_PRODUCER", "Wartet auf Produzent"),
    ("WAITING_FOR_CONSUMER", "Wartet auf Verbraucher"),
    ("ACTIVE", "Aktiv"),
    ("TERMINATED", "Beendet"),
    ("ERROR", "Fehler"),
];
const PART_STATE: &[(&str, &str)] = &[
    ("DRAFT", "Entwurf"),
    ("WAITING_FOR_FOUNDER_APPROVAL", "Wartet auf Gründer"),
    ("WAITING_FOR_VNB_APPROVAL", "Wartet auf Netzbetreiber"),
    ("WAITING_FOR_SIGNATURE", "Wartet auf Unterschrift"),
    ("WAITING_FOR_PRODUCER", "Wartet auf Produzent"),
    ("WAITING_FOR_CONSUMER", "Wartet auf Verbraucher"),
    ("ACTIVE", "Aktiv"),
    ("TERMINATED", "Beendet"),
    ("ERROR", "Fehler"),
];

const LANGUAGE: &[(&str, &str)] = &[
    ("DE", "Deutsch"),
    ("FR", "Französisch"),
    ("IT", "Italienisch"),
    ("EN", "Englisch"),
];

static PATCH_FIELDS_FOUNDING: &[PatchField] = &[
    PatchField { path: "identity.name", label: "Name der Gründung", kind: ValueKind::Text },
    PatchField { path: "identity.externalLegId", label: "Externe Gründungs-ID", kind: ValueKind::Text },
    PatchField { path: "visibility.legVisibility", label: "Sichtbarkeit", kind: ValueKind::Choice(VISIBILITY) },
    PatchField { path: "visibility.listFullAddress", label: "Vollständige Adresse öffentlich anzeigen", kind: ValueKind::Bool },
    PatchField { path: "skipScSignProcess", label: "Unterschrifts-Prozess (SC Sign) überspringen", kind: ValueKind::Bool },
    PatchField { path: "ownership.languageCode", label: "Sprache", kind: ValueKind::Choice(LANGUAGE) },
    PatchField { path: "ownership.newsletterConsent", label: "Newsletter-Zustimmung", kind: ValueKind::Bool },
    PatchField { path: "lifecycle.startingDateAsap", label: "Start sobald wie möglich", kind: ValueKind::Bool },
    PatchField { path: "lifecycle.startingDate", label: "Startdatum", kind: ValueKind::Date },
    PatchField { path: "lifecycle.minimumTermMonths", label: "Mindestlaufzeit (Monate)", kind: ValueKind::Number },
    PatchField { path: "energyConfiguration.discountLevel", label: "Rabattstufe", kind: ValueKind::Choice(DISCOUNT) },
    PatchField { path: "energyConfiguration.energyModelState", label: "Status", kind: ValueKind::Choice(LEG_STATE) },
    PatchField { path: "energyConfiguration.gridConnection.gridLevel", label: "Netzebene", kind: ValueKind::Choice(GRID_LEVEL) },
    PatchField { path: "energyConfiguration.gridConnection.transformerStation", label: "Trafostation", kind: ValueKind::Text },
    PatchField { path: "energyConfiguration.gridConnection.substation", label: "Unterwerk", kind: ValueKind::Text },
    PatchField { path: "energyConfiguration.gridConnection.networkOperator", label: "Netzbetreiber", kind: ValueKind::Text },
    PatchField { path: "energyConfiguration.gridConnection.address.street", label: "Referenzadresse: Strasse", kind: ValueKind::Text },
    PatchField { path: "energyConfiguration.gridConnection.address.houseNumber", label: "Referenzadresse: Hausnummer", kind: ValueKind::Text },
    PatchField { path: "energyConfiguration.gridConnection.address.zipCode", label: "Referenzadresse: PLZ", kind: ValueKind::Text },
    PatchField { path: "energyConfiguration.gridConnection.address.city", label: "Referenzadresse: Ort", kind: ValueKind::Text },
    PatchField { path: "billingConfiguration.billingType", label: "Abrechnungsart", kind: ValueKind::Choice(BILLING_TYPE) },
    PatchField { path: "billingConfiguration.pricingModel", label: "Preismodell", kind: ValueKind::Choice(PRICING_MODEL) },
    PatchField { path: "billingConfiguration.individualFixedPrice", label: "Individueller Fixpreis", kind: ValueKind::Number },
    PatchField { path: "billingConfiguration.standardTariffDiscount", label: "Rabatt Standardtarif", kind: ValueKind::Number },
    PatchField { path: "billingConfiguration.bankAccount.iban", label: "IBAN", kind: ValueKind::Text },
    PatchField { path: "billingConfiguration.bankAccount.accountHolder", label: "Kontoinhaber", kind: ValueKind::Text },
    PatchField { path: "billingConfiguration.billingServiceProvider", label: "Abrechnungsdienstleister", kind: ValueKind::Text },
    PatchField { path: "billingConfiguration.billingAddress.street", label: "Rechnungsadresse: Strasse", kind: ValueKind::Text },
    PatchField { path: "billingConfiguration.billingAddress.houseNumber", label: "Rechnungsadresse: Hausnummer", kind: ValueKind::Text },
    PatchField { path: "billingConfiguration.billingAddress.zipCode", label: "Rechnungsadresse: PLZ", kind: ValueKind::Text },
    PatchField { path: "billingConfiguration.billingAddress.city", label: "Rechnungsadresse: Ort", kind: ValueKind::Text },
    PatchField { path: "participation.allowedParticipants", label: "Erlaubte Teilnehmer", kind: ValueKind::Choice(PARTICIPANT_TYPE) },
    PatchField { path: "ownership.founder.externalFounderId", label: "Gründer: Externe ID", kind: ValueKind::Text },
    PatchField { path: "ownership.founder.contact.firstName", label: "Gründer: Vorname", kind: ValueKind::Text },
    PatchField { path: "ownership.founder.contact.lastName", label: "Gründer: Nachname", kind: ValueKind::Text },
    PatchField { path: "ownership.founder.contact.dateOfBirth", label: "Gründer: Geburtsdatum", kind: ValueKind::LocalDate },
    PatchField { path: "ownership.founder.contact.email", label: "Gründer: E-Mail", kind: ValueKind::Text },
    PatchField { path: "ownership.founder.contact.phone", label: "Gründer: Telefon", kind: ValueKind::Text },
    PatchField { path: "ownership.founder.contact.companyName", label: "Gründer: Firma", kind: ValueKind::Text },
    PatchField { path: "ownership.founder.contact.uidNumber", label: "Gründer: UID-Nummer", kind: ValueKind::Text },
    PatchField { path: "ownership.founder.contact.businessPartnerNumber", label: "Gründer: Geschäftspartnernummer", kind: ValueKind::Text },
    PatchField { path: "ownership.founder.contact.legalForm", label: "Gründer: Rechtsform", kind: ValueKind::Choice(LEGAL_FORM) },
    PatchField { path: "ownership.founder.address.street", label: "Gründer: Strasse", kind: ValueKind::Text },
    PatchField { path: "ownership.founder.address.houseNumber", label: "Gründer: Hausnummer", kind: ValueKind::Text },
    PatchField { path: "ownership.founder.address.zipCode", label: "Gründer: PLZ", kind: ValueKind::Text },
    PatchField { path: "ownership.founder.address.city", label: "Gründer: Ort", kind: ValueKind::Text },
];

static PATCH_FIELDS_PARTICIPATION: &[PatchField] = &[
    PatchField { path: "externalParticipantId", label: "Externe Teilnehmer-ID", kind: ValueKind::Text },
    PatchField { path: "participantType", label: "Art der Teilnahme", kind: ValueKind::Choice(PARTICIPANT_TYPE) },
    PatchField { path: "participationState", label: "Status", kind: ValueKind::Choice(PART_STATE) },
    PatchField { path: "startingDateAsap", label: "Start sobald wie möglich", kind: ValueKind::Bool },
    PatchField { path: "startingDate", label: "Startdatum", kind: ValueKind::Date },
    PatchField { path: "contact.firstName", label: "Vorname", kind: ValueKind::Text },
    PatchField { path: "contact.lastName", label: "Nachname", kind: ValueKind::Text },
    PatchField { path: "contact.dateOfBirth", label: "Geburtsdatum", kind: ValueKind::LocalDate },
    PatchField { path: "contact.email", label: "E-Mail", kind: ValueKind::Text },
    PatchField { path: "contact.phone", label: "Telefon", kind: ValueKind::Text },
    PatchField { path: "contact.companyName", label: "Firma", kind: ValueKind::Text },
    PatchField { path: "contact.uidNumber", label: "UID-Nummer", kind: ValueKind::Text },
    PatchField { path: "contact.businessPartnerNumber", label: "Geschäftspartnernummer", kind: ValueKind::Text },
    PatchField { path: "contact.legalForm", label: "Rechtsform", kind: ValueKind::Choice(LEGAL_FORM) },
    PatchField { path: "address.street", label: "Strasse", kind: ValueKind::Text },
    PatchField { path: "address.houseNumber", label: "Hausnummer", kind: ValueKind::Text },
    PatchField { path: "address.zipCode", label: "PLZ", kind: ValueKind::Text },
    PatchField { path: "address.city", label: "Ort", kind: ValueKind::Text },
];

/// Liefert die änderbaren Felder für eine "anpassen"-Aktion (sonst leer).
fn patch_fields_for(path_suffix: &str) -> &'static [PatchField] {
    match path_suffix {
        "patchFounding" => PATCH_FIELDS_FOUNDING,
        "patchParticipation" => PATCH_FIELDS_PARTICIPATION,
        _ => &[],
    }
}

fn find_patch_field(fields: &'static [PatchField], path: &str) -> Option<&'static PatchField> {
    fields.iter().find(|f| f.path == path)
}

/// Standardwert beim Hinzufügen eines Felds (erster Auswahlwert bzw. leer).
fn default_value_for(kind: ValueKind) -> String {
    match kind {
        ValueKind::Bool => "false".to_string(),
        ValueKind::Choice(opts) => opts.first().map(|(v, _)| v.to_string()).unwrap_or_default(),
        _ => String::new(),
    }
}

/// Setzt einen Wert unter einem Punkt-Pfad in ein verschachteltes JSON-Objekt.
fn insert_nested(root: &mut serde_json::Map<String, serde_json::Value>, path: &str, value: serde_json::Value) {
    let parts: Vec<&str> = path.split('.').collect();
    let mut cur = root;
    for part in &parts[..parts.len() - 1] {
        let entry = cur
            .entry((*part).to_string())
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
        if !entry.is_object() {
            *entry = serde_json::Value::Object(serde_json::Map::new());
        }
        cur = entry.as_object_mut().unwrap();
    }
    cur.insert(parts[parts.len() - 1].to_string(), value);
}

// ----------------------------------------------------------------------------
// Einstellungen (werden lokal gespeichert)
// ----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
enum Env {
    Test,
    Prod,
}

impl Env {
    fn host(self) -> &'static str {
        match self {
            Env::Test => "https://test.industry.leg.digital.deals",
            Env::Prod => "https://industry.leg.digital.deals",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Env::Test => "Test",
            Env::Prod => "Produktion",
        }
    }
    /// Adresse des Servers; in Debug-Builds per `LEGHUB_SUPPORT_HOST` überschreibbar.
    fn base_url(self) -> String {
        host_override().unwrap_or_else(|| self.host().to_string())
    }
}

/// Nur für lokale Tests: `LEGHUB_SUPPORT_HOST=http://localhost:4502 dx serve`.
/// In Release-Builds immer `None`.
fn host_override() -> Option<String> {
    if !cfg!(debug_assertions) {
        return None;
    }
    std::env::var("LEGHUB_SUPPORT_HOST")
        .ok()
        .map(|h| h.trim().trim_end_matches('/').to_string())
        .filter(|h| !h.is_empty())
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Settings {
    env: Env,
    /// Zugangsschlüssel für die Test-Umgebung.
    #[serde(default)]
    test_key: String,
    /// Zugangsschlüssel für die Produktiv-Umgebung.
    #[serde(default)]
    prod_key: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { env: Env::Test, test_key: String::new(), prod_key: String::new() }
    }
}

impl Settings {
    /// Zugangsschlüssel der aktuell gewählten Umgebung.
    fn key(&self) -> &str {
        match self.env {
            Env::Test => &self.test_key,
            Env::Prod => &self.prod_key,
        }
    }
}

fn settings_path() -> Option<std::path::PathBuf> {
    let dirs = directories::ProjectDirs::from("com", "ajila", "LegHub Support-Tool")?;
    Some(dirs.config_dir().join("settings.json"))
}

fn load_settings() -> Settings {
    settings_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_settings(settings: &Settings) {
    if let Some(path) = settings_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(settings) {
            let _ = std::fs::write(path, json);
        }
    }
}

// ----------------------------------------------------------------------------
// Ergebnis einer Aktion
// ----------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
enum ResultKind {
    Success,
    /// Weder klar erfolgreich noch fehlgeschlagen (z. B. Zeitüberschreitung beim Löschen).
    Warning,
    Error,
}

#[derive(Clone, PartialEq)]
struct ActionResult {
    kind: ResultKind,
    message: String,
}

/// Inhalt der Anfrage: entweder klassische Formularfelder oder ein JSON-Text.
#[derive(Clone, PartialEq)]
enum RequestBody {
    Form(Vec<(String, String)>),
    Json(String),
}

/// Vorbereitete Anfrage, die erst nach Bestätigung im Popup gesendet wird.
#[derive(Clone, PartialEq)]
struct PendingRequest {
    url: String,
    key: String,
    query: Vec<(String, String)>,
    body: RequestBody,
    action_title: String,
    env: Env,
    /// Bei einer echten Löschung: die Zusammenfassung aus dem vorangegangenen Probelauf.
    deletion: Option<DeletionSummary>,
}

// ----------------------------------------------------------------------------
// App
// ----------------------------------------------------------------------------

/// App-Icon als rohe RGBA-Daten (128×128), zur Laufzeit ins Fenster gesetzt.
/// (Auf macOS liefert das gebündelte .icns das Dock-Icon; unter Windows/Linux
/// zeigt das Fenster-/Taskleisten-Icon dieses Bild.)
const ICON_RGBA: &[u8] = include_bytes!("../assets/icons/icon-128.rgba");

fn window_icon() -> Option<dioxus::desktop::tao::window::Icon> {
    dioxus::desktop::tao::window::Icon::from_rgba(ICON_RGBA.to_vec(), 128, 128).ok()
}

fn main() {
    let window = WindowBuilder::new()
        .with_title("LegHub Support-Tool")
        .with_window_icon(window_icon());
    dioxus::LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(window))
        .launch(App);
}

#[component]
fn App() -> Element {
    let mut settings = use_signal(load_settings);
    let mut selected = use_signal(|| 0usize);
    let mut values = use_signal(HashMap::<String, String>::new);
    let mut sending = use_signal(|| false);
    let mut result = use_signal(|| Option::<ActionResult>::None);
    let mut show_settings = use_signal(|| false);
    let mut pending = use_signal(|| Option::<PendingRequest>::None);
    let mut confirm_checked = use_signal(|| false);
    // Ausgewählte Änderungen für "anpassen"-Aktionen: (Pfad, Wert).
    let mut patch_changes = use_signal(Vec::<(String, String)>::new);
    // Übersicht mit dem Ablauf-Diagramm ist die Einstiegsseite.
    let mut show_overview = use_signal(|| true);
    // Lösch-Aktionen: letzter erfolgreicher Probelauf (nur dann darf wirklich gelöscht werden),
    // Ergebnis der echten Löschung samt Umgebung und die Option «Schutz übersteuern».
    let mut dry_run = use_signal(|| Option::<DryRun>::None);
    let mut deleted = use_signal(|| Option::<(DeletionSummary, Env)>::None);
    let mut force = use_signal(|| false);
    let mut offer_force = use_signal(|| false);
    let mut copied = use_signal(|| false);

    // Wechselt zu einer Aktion und leert die zuvor erfassten Eingaben.
    let mut select_action = move |i: usize| {
        selected.set(i);
        values.write().clear();
        patch_changes.write().clear();
        result.set(None);
        dry_run.set(None);
        deleted.set(None);
        force.set(false);
        offer_force.set(false);
        show_overview.set(false);
    };

    let action = &ACTIONS[selected()];
    let patch_fields = patch_fields_for(action.path_suffix);
    let is_delete = is_delete_action(action.path_suffix);

    rsx! {
        document::Stylesheet { href: STYLE }
        div { class: "app",
            // -------- Seitenpanel --------
            aside { class: "sidebar",
                div { class: "brand",
                    img { class: "logo", src: LOGO, alt: "LEGhub" }
                    div { class: "brand-sub", "Support-Tool" }
                }
                div { class: "nav",
                    button {
                        class: if show_overview() { "nav-item active" } else { "nav-item" },
                        onclick: move |_| show_overview.set(true),
                        "Übersicht"
                    }
                    for group in GROUPS.iter() {
                        div { class: "nav-group", "{group}" }
                        for (i , a) in ACTIONS.iter().enumerate() {
                            if a.group == *group {
                                button {
                                    class: if selected() == i && !show_overview() { "nav-item active" } else { "nav-item" },
                                    onclick: move |_| select_action(i),
                                    "{a.title}"
                                }
                            }
                        }
                    }
                }
                button {
                    class: "settings-btn",
                    onclick: move |_| show_settings.set(true),
                    "⚙  Einstellungen"
                }
            }

            // -------- Hauptbereich --------
            main { class: if show_overview() { "content content-wide" } else { "content" },
                div { class: "env-badge",
                    "Umgebung: "
                    span { class: if settings.read().env == Env::Prod { "env-prod" } else { "env-test" },
                        "{settings.read().env.label()}"
                    }
                    if let Some(host) = host_override() {
                        span { class: "env-override", "Host überschrieben: {host}" }
                    }
                }

                if show_overview() {
                    Overview {
                        on_open: move |path_suffix: &'static str| {
                            if let Some(i) = ACTIONS.iter().position(|a| a.path_suffix == path_suffix) {
                                select_action(i);
                            }
                        },
                    }
                } else {
                    h1 { "{action.title}" }
                    p { class: "description", "{action.description}" }
                    if let Some(guide) = delete_guide(action) {
                        {delete_guide_view(guide)}
                    }

                    form { onsubmit: move |e| e.stop_propagation(),
                        for field in action.fields.iter() {
                            div { class: "field",
                                label {
                                    "{field.label}"
                                    if !field.required {
                                        span { class: "optional", " (optional)" }
                                    }
                                }
                                if is_multi_value(field.key) {
                                    textarea {
                                        rows: "3",
                                        placeholder: "CH1015001234500010060400007001001\nCH1015001234500010060400007001002",
                                        autocomplete: "off",
                                        value: "{values.read().get(field.key).cloned().unwrap_or_default()}",
                                        oninput: move |e| {
                                            values.write().insert(field.key.to_string(), e.value());
                                            // Ein Probelauf gilt nur für genau die geprüften Eingaben.
                                            dry_run.set(None);
                                            deleted.set(None);
                                        },
                                    }
                                } else {
                                    input {
                                        r#type: "text",
                                        autocomplete: "off",
                                        value: "{values.read().get(field.key).cloned().unwrap_or_default()}",
                                        oninput: move |e| {
                                            values.write().insert(field.key.to_string(), e.value());
                                            dry_run.set(None);
                                            deleted.set(None);
                                        },
                                    }
                                }
                                if !field.hint.is_empty() {
                                    div { class: "hint", "{field.hint}" }
                                }
                            }
                        }

                        // -------- Feld-Auswahl für "anpassen"-Aktionen --------
                        if !patch_fields.is_empty() {
                            div { class: "field",
                                label { "Zu ändernde Angaben" }
                                div { class: "hint",
                                    "Wählen Sie unten die Felder aus, die geändert werden sollen."
                                }

                                div { class: "patch-list",
                                    for (idx , change) in patch_changes.read().iter().enumerate() {
                                        if let Some(pf) = find_patch_field(patch_fields, &change.0) {
                                            div { class: "patch-row",
                                                div { class: "patch-row-label", "{pf.label}" }
                                                div { class: "patch-row-value",
                                                    match pf.kind {
                                                        ValueKind::Bool => rsx! {
                                                            select {
                                                                value: "{change.1}",
                                                                onchange: move |e| {
                                                                    if let Some(c) = patch_changes.write().get_mut(idx) {
                                                                        c.1 = e.value();
                                                                    }
                                                                },
                                                                option { value: "true", "Ja" }
                                                                option { value: "false", "Nein" }
                                                            }
                                                        },
                                                        ValueKind::Choice(opts) => rsx! {
                                                            select {
                                                                value: "{change.1}",
                                                                onchange: move |e| {
                                                                    if let Some(c) = patch_changes.write().get_mut(idx) {
                                                                        c.1 = e.value();
                                                                    }
                                                                },
                                                                for (v , l) in opts.iter() {
                                                                    option { value: "{v}", "{l}" }
                                                                }
                                                            }
                                                        },
                                                        ValueKind::Number => rsx! {
                                                            input {
                                                                r#type: "number",
                                                                value: "{change.1}",
                                                                oninput: move |e| {
                                                                    if let Some(c) = patch_changes.write().get_mut(idx) {
                                                                        c.1 = e.value();
                                                                    }
                                                                },
                                                            }
                                                        },
                                                        ValueKind::Date | ValueKind::LocalDate => rsx! {
                                                            input {
                                                                r#type: "date",
                                                                value: "{change.1}",
                                                                oninput: move |e| {
                                                                    if let Some(c) = patch_changes.write().get_mut(idx) {
                                                                        c.1 = e.value();
                                                                    }
                                                                },
                                                            }
                                                        },
                                                        ValueKind::Text => rsx! {
                                                            input {
                                                                r#type: "text",
                                                                autocomplete: "off",
                                                                value: "{change.1}",
                                                                oninput: move |e| {
                                                                    if let Some(c) = patch_changes.write().get_mut(idx) {
                                                                        c.1 = e.value();
                                                                    }
                                                                },
                                                            }
                                                        },
                                                    }
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "patch-remove",
                                                    title: "Entfernen",
                                                    onclick: move |_| {
                                                        patch_changes.write().remove(idx);
                                                    },
                                                    "✕"
                                                }
                                            }
                                        }
                                    }
                                }

                                select {
                                    class: "patch-add",
                                    value: "",
                                    onchange: move |e| {
                                        let path = e.value();
                                        if path.is_empty() {
                                            return;
                                        }
                                        if let Some(pf) = find_patch_field(patch_fields, &path) {
                                            let already = patch_changes.read().iter().any(|c| c.0 == path);
                                            if !already {
                                                patch_changes.write().push((path.clone(), default_value_for(pf.kind)));
                                            }
                                        }
                                    },
                                    option { value: "", "＋ Feld hinzufügen …" }
                                    for pf in patch_fields.iter() {
                                        if !patch_changes.read().iter().any(|c| c.0 == pf.path) {
                                            option { value: "{pf.path}", "{pf.label}" }
                                        }
                                    }
                                }
                            }
                        }

                        // -------- Schutz übersteuern (erst nach einer Schutzregel-Meldung) --------
                        if is_delete && (offer_force() || force()) {
                            label { class: "confirm-check force-check",
                                input {
                                    r#type: "checkbox",
                                    checked: force(),
                                    onchange: move |e| {
                                        force.set(e.checked());
                                        dry_run.set(None);
                                        deleted.set(None);
                                    },
                                }
                                span {
                                    strong { "Schutz übersteuern (force)" }
                                    br {}
                                    "Nur aktivieren, wenn die Löschung trotz der Meldung oben richtig ist (siehe «Wann ist «Schutz übersteuern» nötig?»). Sonst ist meist Austritt bzw. Auflösung der richtige Weg. Nach dem Aktivieren erneut einen Probelauf starten. Löschen Sie Daten, die der Netzbetreiber kennt, müssen Sie ihn separat informieren."
                                }
                            }
                        }

                        button {
                            r#type: "button",
                            class: "run-btn",
                            disabled: sending(),
                            onclick: move |_| {
                                let action = &ACTIONS[selected()];
                                let is_delete = is_delete_action(action.path_suffix);
                                let current = settings.read().clone();

                                // Zugangsschlüssel für die gewählte Umgebung vorhanden?
                                if current.key().trim().is_empty() {
                                    result
                                        .set(
                                            Some(ActionResult {
                                                kind: ResultKind::Error,
                                                message: format!(
                                                    "Für die Umgebung \"{}\" ist kein Zugangsschlüssel hinterlegt. Bitte in den Einstellungen eintragen.",
                                                    current.env.label(),
                                                ),
                                            }),
                                        );
                                    show_settings.set(true);
                                    return;
                                }
                                let is_patch = !patch_fields.is_empty();
                                let vals = values.read();
                                let mut query: Vec<(String, String)> = Vec::new();
                                let mut form: Vec<(String, String)> = Vec::new();
                                for f in action.fields.iter() {
                                    let v = vals.get(f.key).map(|s| s.trim().to_string()).unwrap_or_default();
                                    if is_multi_value(f.key) {
                                        let entries = split_multi_value(&v);
                                        if f.required && entries.is_empty() {
                                            result
                                                .set(
                                                    Some(ActionResult {
                                                        kind: ResultKind::Error,
                                                        message: "Bitte alle Pflichtfelder ausfüllen.".into(),
                                                    }),
                                                );
                                            return;
                                        }
                                        form.extend(entries.into_iter().map(|e| (f.key.to_string(), e)));
                                        continue;
                                    }
                                    let v = if f.key == "ticket" { v.to_uppercase() } else { v };
                                    if f.required && v.is_empty() {
                                        result
                                            .set(
                                                Some(ActionResult {
                                                    kind: ResultKind::Error,
                                                    message: "Bitte alle Pflichtfelder ausfüllen.".into(),
                                                }),
                                            );
                                        return;
                                    }
                                    if v.is_empty() {
                                        continue;
                                    }
                                    if is_patch {
                                        query.push((f.key.to_string(), v));
                                    } else {
                                        form.push((f.key.to_string(), v));
                                    }
                                }
                                drop(vals);
                                if is_delete {
                                    let ticket = form
                                        .iter()
                                        .find(|(k, _)| k == "ticket")
                                        .map(|(_, v)| v.as_str())
                                        .unwrap_or_default();
                                    if !is_valid_ticket(ticket) {
                                        result
                                            .set(
                                                Some(ActionResult {
                                                    kind: ResultKind::Error,
                                                    message: "Bitte eine gültige Ticket-Nummer angeben, z. B. LHSD-1091."
                                                        .into(),
                                                }),
                                            );
                                        return;
                                    }
                                    if force() {
                                        form.push(("force".into(), "true".into()));
                                    }
                                }
                                let body = if is_patch {
                                    let changes = patch_changes.read();
                                    if changes.is_empty() {
                                        result
                                            .set(
                                                Some(ActionResult {
                                                    kind: ResultKind::Error,
                                                    message: "Bitte mindestens ein zu änderndes Feld auswählen."
                                                        .into(),
                                                }),

                                            );
                                        return;
                                    }
                                    let mut root = serde_json::Map::new();
                                    for (path, raw) in changes.iter() {
                                        let Some(pf) = find_patch_field(patch_fields, path) else { continue };
                                        let value = raw.trim();
                                        let json_value = match pf.kind {
                                            ValueKind::Bool => serde_json::Value::Bool(value == "true"),
                                            ValueKind::Number => {
                                                let normalized = value.replace(',', ".");
                                                if let Ok(i) = normalized.parse::<i64>() {
                                                    serde_json::json!(i)
                                                } else if let Ok(n) = normalized.parse::<f64>() {
                                                    serde_json::json!(n)
                                                } else {
                                                    result
                                                        .set(
                                                            Some(ActionResult {
                                                                kind: ResultKind::Error,
                                                                message: format!(
                                                                    "Bei \"{}\" bitte eine gültige Zahl eingeben.",
                                                                    pf.label,
                                                                ),
                                                            }),
                                                        );
                                                    return;
                                                }
                                            }
                                            ValueKind::Date => {
                                                if value.is_empty() {
                                                    result
                                                        .set(
                                                            Some(ActionResult {
                                                                kind: ResultKind::Error,
                                                                message: format!(
                                                                    "Bei \"{}\" bitte ein Datum wählen.",
                                                                    pf.label,
                                                                ),
                                                            }),
                                                        );
                                                    return;
                                                }
                                                serde_json::Value::String(format!("{value}T00:00:00Z"))
                                            }
                                            _ => {
                                                if value.is_empty() {
                                                    result
                                                        .set(
                                                            Some(ActionResult {
                                                                kind: ResultKind::Error,
                                                                message: format!(
                                                                    "Bei \"{}\" bitte einen Wert angeben.",
                                                                    pf.label,
                                                                ),
                                                            }),
                                                        );
                                                    return;
                                                }
                                                serde_json::Value::String(value.to_string())
                                            }
                                        };
                                        insert_nested(&mut root, path, json_value);
                                    }
                                    drop(changes);
                                    RequestBody::Json(serde_json::Value::Object(root).to_string())
                                } else {
                                    RequestBody::Form(form)
                                };
                                let url = format!(
                                    "{}/bin/com/ajila/forms/industry/leg/{}",
                                    current.env.base_url(),
                                    action.path_suffix
                                );
                                let key = current.key().trim().to_string();
                                result.set(None);
                                let req = PendingRequest {
                                    url,
                                    key,
                                    query,
                                    body,
                                    action_title: action.title.to_string(),
                                    env: current.env,
                                    deletion: None,
                                };
                                if is_delete {
                                    // Probelauf: ohne dryRun-Parameter (Server-Default = nur anzeigen),
                                    // daher ohne Bestätigung.
                                    dry_run.set(None);
                                    deleted.set(None);
                                    copied.set(false);
                                    sending.set(true);
                                    spawn(async move {
                                        let outcome = send_delete_request(&req).await;
                                        apply_delete_outcome(outcome, req, result, dry_run, deleted, offer_force);
                                        sending.set(false);
                                    });
                                    return;
                                }
                                confirm_checked.set(false);
                                pending.set(Some(req));
                            },
                            if sending() {
                                "Wird ausgeführt …"
                            } else if is_delete {
                                "Probelauf starten"
                            } else {
                                "Ausführen"
                            }
                        }
                    }

                    if let Some(res) = result() {
                        div {
                            class: match res.kind {
                                ResultKind::Success => "result result-ok",
                                ResultKind::Warning => "result result-warn",
                                ResultKind::Error => "result result-err",
                            },
                            "{res.message}"
                        }
                    }

                    // -------- Probelauf-Ergebnis: erst hier wird echtes Löschen möglich --------
                    if is_delete {
                        if let Some(dr) = dry_run() {
                            div { class: "result result-dry",
                                DeletionSummaryView { summary: dr.summary.clone() }
                                p { class: "summary-note",
                                    "Bitte prüfen. Gelöscht wird genau das oben Aufgeführte. Wird eine Eingabe geändert, muss erneut ein Probelauf gemacht werden."
                                }
                                button {
                                    r#type: "button",
                                    class: "danger-btn",
                                    disabled: sending(),
                                    onclick: move |_| {
                                        if let Some(dr) = dry_run() {
                                            // Genau den geprüften Request verwenden, nicht die aktuellen Felder.
                                            let mut req = dr.request;
                                            if let RequestBody::Form(form) = &mut req.body {
                                                form.push(("dryRun".into(), "false".into()));
                                            }
                                            req.deletion = Some(dr.summary);
                                            confirm_checked.set(false);
                                            pending.set(Some(req));
                                        }
                                    },
                                    "Jetzt wirklich löschen"
                                }
                            }
                        }

                        if let Some((summary, env)) = deleted() {
                            div { class: "result result-ok",
                                DeletionSummaryView { summary: summary.clone() }
                                p { class: "summary-note",
                                    "Bitte die Zusammenfassung ins Ticket kopieren."
                                }
                                button {
                                    r#type: "button",
                                    class: "cancel-btn",
                                    onclick: move |_| {
                                        copy_to_clipboard(&summary.to_ticket_text(env));
                                        copied.set(true);
                                    },
                                    if copied() {
                                        "Kopiert ✓"
                                    } else {
                                        "Kopieren"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // -------- Einstellungen (Overlay) --------
            if show_settings() {
                div {
                    class: "overlay",
                    onclick: move |_| show_settings.set(false),
                    div {
                        class: "modal",
                        onclick: move |e| e.stop_propagation(),
                        h2 { "Einstellungen" }

                        div { class: "field",
                            label { "Umgebung" }
                            div { class: "env-switch",
                                button {
                                    r#type: "button",
                                    class: if settings.read().env == Env::Test { "env-opt active" } else { "env-opt" },
                                    onclick: move |_| {
                                        settings.write().env = Env::Test;
                                        save_settings(&settings.read());
                                        dry_run.set(None);
                                        deleted.set(None);
                                    },
                                    "Test"
                                }
                                button {
                                    r#type: "button",
                                    class: if settings.read().env == Env::Prod { "env-opt active env-opt-prod" } else { "env-opt" },
                                    onclick: move |_| {
                                        settings.write().env = Env::Prod;
                                        save_settings(&settings.read());
                                        dry_run.set(None);
                                        deleted.set(None);
                                    },
                                    "Produktion"
                                }
                            }
                        }

                        if settings.read().env == Env::Test {
                            div { class: "field",
                                label { "Zugangsschlüssel Test" }
                                input {
                                    r#type: "password",
                                    autocomplete: "off",
                                    value: "{settings.read().test_key}",
                                    oninput: move |e| {
                                        settings.write().test_key = e.value();
                                    },
                                }
                            }
                        }

                        if settings.read().env == Env::Prod {
                            div { class: "field",
                                label { "Zugangsschlüssel Produktion" }
                                input {
                                    r#type: "password",
                                    autocomplete: "off",
                                    value: "{settings.read().prod_key}",
                                    oninput: move |e| {
                                        settings.write().prod_key = e.value();
                                    },
                                }
                            }
                        }

                        button {
                            r#type: "button",
                            class: "run-btn",
                            onclick: move |_| {
                                save_settings(&settings.read());
                                show_settings.set(false);
                            },
                            "Speichern"
                        }
                    }
                }
            }

            // -------- Bestätigung (Overlay) --------
            if let Some(req) = pending() {
                div { class: "overlay",
                    div {
                        class: "modal",
                        onclick: move |e| e.stop_propagation(),
                        h2 { "Aktion bestätigen" }
                        p { class: "confirm-line",
                            "Aktion: "
                            strong { "{req.action_title}" }
                        }
                        p { class: "confirm-line",
                            "Umgebung: "
                            span { class: if req.env == Env::Prod { "env-prod" } else { "env-test" },
                                "{req.env.label()}"
                            }
                        }
                        if req.env == Env::Prod {
                            p { class: "confirm-warn",
                                "Achtung: Diese Aktion wirkt sofort auf echte Daten in der Produktion."
                            }
                        }
                        if let Some(d) = &req.deletion {
                            p { class: "confirm-line",
                                "Ticket: "
                                strong { "{d.ticket}" }
                            }
                            p { class: "confirm-line",
                                "Es wird endgültig gelöscht: "
                                strong { "{d.short()}" }
                            }
                            if d.forced {
                                p { class: "confirm-warn",
                                    "Schutz übersteuert: Der Netzbetreiber wird nicht automatisch informiert. Bitte separat benachrichtigen."
                                }
                            }
                        }

                        label { class: "confirm-check",
                            input {
                                r#type: "checkbox",
                                checked: confirm_checked(),
                                onchange: move |e| confirm_checked.set(e.checked()),
                            }
                            if req.deletion.is_some() {
                                span { "Ja, ich habe den Probelauf geprüft und möchte jetzt endgültig löschen." }
                            } else {
                                span { "Ja, ich möchte diese Aktion jetzt ausführen." }
                            }
                        }

                        div { class: "confirm-actions",
                            button {
                                r#type: "button",
                                class: "cancel-btn",
                                onclick: move |_| {
                                    pending.set(None);
                                    confirm_checked.set(false);
                                },
                                "Abbrechen"
                            }
                            button {
                                r#type: "button",
                                class: if req.deletion.is_some() { "danger-btn" } else { "run-btn" },
                                disabled: !confirm_checked(),
                                onclick: move |_| {
                                    if let Some(req) = pending() {
                                        pending.set(None);
                                        confirm_checked.set(false);
                                        sending.set(true);
                                        result.set(None);
                                        spawn(async move {
                                            if req.deletion.is_some() {
                                                let outcome = send_delete_request(&req).await;
                                                apply_delete_outcome(outcome, req, result, dry_run, deleted, offer_force);
                                            } else {
                                                let outcome = send_request(req.url, req.key, req.query, req.body).await;
                                                result.set(Some(outcome));
                                            }
                                            sending.set(false);
                                        });
                                    }
                                },
                                if req.deletion.is_some() {
                                    "Endgültig löschen"
                                } else {
                                    "Ausführen"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Führt die Anfrage aus und liefert eine verständliche Rückmeldung.
async fn send_request(
    url: String,
    key: String,
    query: Vec<(String, String)>,
    body: RequestBody,
) -> ActionResult {
    let client = reqwest::Client::new();
    // Die "anpassen"-Endpunkte (JSON-Body) erwarten HTTP PATCH, alle anderen POST.
    let mut builder = match &body {
        RequestBody::Form(_) => client.post(&url),
        RequestBody::Json(_) => client.patch(&url),
    };
    builder = builder.header("X-Api-Key", key);
    if !query.is_empty() {
        builder = builder.query(&query);
    }
    builder = match body {
        RequestBody::Form(form) => builder.form(&form),
        RequestBody::Json(text) => builder.header("Content-Type", "application/json").body(text),
    };
    match builder.send().await {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                ActionResult {
                    kind: ResultKind::Success,
                    message: "Erledigt ✓  Die Aktion wurde erfolgreich ausgeführt.".into(),
                }
            } else if status.as_u16() == 403 {
                key_error()
            } else {
                generic_error()
            }
        }
        Err(_) => network_error(),
    }
}

fn error(message: impl Into<String>) -> ActionResult {
    ActionResult { kind: ResultKind::Error, message: message.into() }
}

fn key_error() -> ActionResult {
    error("Der Zugangsschlüssel fehlt oder ist falsch. Bitte in den Einstellungen prüfen.")
}

fn generic_error() -> ActionResult {
    error("Die Aktion konnte nicht ausgeführt werden. Bitte die eingegebene ID prüfen und erneut versuchen.")
}

fn network_error() -> ActionResult {
    error("Der Server ist nicht erreichbar. Bitte Internetverbindung und Umgebung prüfen.")
}

/// Sendet einen Lösch-Aufruf (Probelauf oder echt) und wertet die JSON-Antwort aus.
async fn send_delete_request(req: &PendingRequest) -> DeleteOutcome {
    let failed = |result| DeleteOutcome::Failed { result, can_force: false };
    let RequestBody::Form(form) = &req.body else {
        return failed(generic_error());
    };
    let resp = match reqwest::Client::new()
        .post(&req.url)
        .header("X-Api-Key", &req.key)
        .form(form)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(_) => return failed(network_error()),
    };
    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    interpret_delete_response(status, &body)
}

/// Übersetzt Status und Body einer Lösch-Antwort in eine verständliche Rückmeldung.
fn interpret_delete_response(status: u16, body: &str) -> DeleteOutcome {
    let failed = |result| DeleteOutcome::Failed { result, can_force: false };
    if (200..300).contains(&status) {
        return match serde_json::from_str::<DeletionSummary>(body) {
            Ok(summary) => DeleteOutcome::Summary(summary),
            Err(_) => failed(error(
                "Die Antwort des Servers konnte nicht gelesen werden. Bitte mit einem neuen Probelauf prüfen, ob gelöscht wurde.",
            )),
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
        504 => failed(ActionResult {
            kind: ResultKind::Warning,
            message: format!(
                "Der Server hat nicht rechtzeitig geantwortet. Die Löschung wurde möglicherweise trotzdem ausgeführt. \
                 Bitte zuerst einen neuen Probelauf starten, bevor Sie es erneut versuchen – meldet dieser «nicht gefunden», \
                 ist die Löschung bereits passiert.{}",
                detail.map(|d| format!("\n\nMeldung des Servers: {d}")).unwrap_or_default()
            ),
        }),
        409 => {
            let detail = detail.unwrap_or_else(|| "Eine Schutzregel verhindert das Löschen.".into());
            let manual = detail.contains("manual clean-up");
            let hint = if manual {
                "Dieser Fall muss manuell bereinigt werden – auch «Schutz übersteuern» hilft hier nicht."
            } else {
                "Falls das Löschen trotzdem nötig ist: «Schutz übersteuern» aktivieren und erneut einen Probelauf starten."
            };
            DeleteOutcome::Failed {
                result: error(format!("Löschen nicht möglich: {detail}\n\n{hint}")),
                can_force: !manual,
            }
        }
        _ => failed(match detail {
            Some(d) => error(format!("Die Aktion konnte nicht ausgeführt werden: {d}")),
            None => generic_error(),
        }),
    }
}

/// Übernimmt das Ergebnis eines Lösch-Aufrufs in den Zustand der Oberfläche.
fn apply_delete_outcome(
    outcome: DeleteOutcome,
    request: PendingRequest,
    mut result: Signal<Option<ActionResult>>,
    mut dry_run: Signal<Option<DryRun>>,
    mut deleted: Signal<Option<(DeletionSummary, Env)>>,
    mut offer_force: Signal<bool>,
) {
    match outcome {
        DeleteOutcome::Summary(summary) if summary.dry_run => {
            // Der gespeicherte Request ist immer der Probelauf ohne dryRun-Parameter.
            let mut request = request;
            if let RequestBody::Form(form) = &mut request.body {
                form.retain(|(k, _)| k != "dryRun");
            }
            request.deletion = None;
            dry_run.set(Some(DryRun { request, summary }));
        }
        DeleteOutcome::Summary(summary) => {
            dry_run.set(None);
            deleted.set(Some((summary, request.env)));
        }
        DeleteOutcome::Failed { result: r, can_force } => {
            // Nach jedem Fehler ist ein neuer Probelauf nötig.
            dry_run.set(None);
            if can_force {
                offer_force.set(true);
            }
            result.set(Some(r));
        }
    }
}

/// Kopiert Text in die Zwischenablage (Fallback für WebViews ohne Clipboard-API).
fn copy_to_clipboard(text: &str) {
    let text = serde_json::to_string(text).unwrap_or_default();
    document::eval(&format!(
        r#"(async function(t) {{
            try {{ await navigator.clipboard.writeText(t); return; }} catch (e) {{}}
            const ta = document.createElement("textarea");
            ta.value = t;
            ta.style.position = "fixed";
            ta.style.opacity = "0";
            document.body.appendChild(ta);
            ta.select();
            document.execCommand("copy");
            document.body.removeChild(ta);
        }})({text});"#
    ));
}

/// Ausführliche Erklärung oberhalb des Formulars einer Lösch-Aktion.
fn delete_guide_view(guide: &'static DeleteGuide) -> Element {
    rsx! {
        div { class: "guide",
            div { class: "guide-warn",
                strong { "Löschen ist nicht Austritt. " }
                "Ist eine LEG oder ein Messpunkt beim Netzbetreiber aktiv und soll per Stichtag enden, ist die Auflösung bzw. der Austritt der richtige Weg. Löschen ist für Daten, die es nicht hätte geben dürfen."
            }

            div { class: "guide-grid",
                div { class: "guide-section",
                    h3 { "Wann verwenden" }
                    ul {
                        for item in guide.when.iter() {
                            li { "{item}" }
                        }
                    }
                }
                div { class: "guide-section",
                    h3 { "Was gelöscht wird" }
                    ul {
                        for item in guide.deletes.iter() {
                            li { "{item}" }
                        }
                    }
                }
                div { class: "guide-section",
                    h3 { "Was bestehen bleibt" }
                    ul {
                        for item in guide.keeps.iter() {
                            li { "{item}" }
                        }
                    }
                }
                div { class: "guide-section",
                    h3 { "Ablauf" }
                    ol {
                        li { "«Probelauf starten» – es wird noch nichts gelöscht." }
                        li { "Zusammenfassung prüfen (LEG, Messpunkte, Anzahl Verträge)." }
                        li { "«Jetzt wirklich löschen» und im Popup bestätigen." }
                        li { "Zusammenfassung ins Ticket kopieren." }
                    }
                }
            }

            if !guide.notes.is_empty() {
                ul { class: "guide-notes",
                    for item in guide.notes.iter() {
                        li { "{item}" }
                    }
                }
            }

            div { class: "guide-section guide-force",
                h3 { "Wann ist «Schutz übersteuern» (force) nötig?" }
                p {
                    "Normalerweise schützt der Server Daten, die der Netzbetreiber bereits kennt. Der Probelauf meldet dann «Löschen nicht möglich» und nennt den Grund – typischerweise, wenn"
                }
                ul {
                    for item in guide.force_when.iter() {
                        li { "{item}" }
                    }
                }
                p {
                    "Erst nach einer solchen Meldung erscheint die Option «Schutz übersteuern». Aktivieren Sie sie nur, wenn die Löschung trotzdem richtig ist (z. B. Test-LEG, Duplikat, falsch erfasster Messpunkt); ist der Vorgang beim Netzbetreiber echt, ist Austritt bzw. Auflösung der richtige Weg. Die Übersteuerung wird im Audit vermerkt."
                }
                p { "{guide.force_never}" }
            }

            ul { class: "guide-notes",
                li {
                    "Es wird niemand informiert: weder Netzbetreiber noch innosolv, und es werden keine E-Mails verschickt. Laufende Abläufe werden nicht abgebrochen – Links aus alten E-Mails führen danach zu einer Fehlermeldung."
                }
                li {
                    "Jeder gelöschte Datensatz wird mit Ticket und Begründung im Audit gesichert und kann von der Entwicklung wiederhergestellt werden (Verträge ohne PDF)."
                }
            }
        }
    }
}

/// Zusammenfassung eines Probelaufs bzw. einer ausgeführten Löschung.
#[component]
fn DeletionSummaryView(summary: DeletionSummary) -> Element {
    let title = if summary.dry_run { "Probelauf – es würden gelöscht:" } else { "Gelöscht ✓" };
    rsx! {
        div { class: "summary",
            div { class: "summary-title", "{title}" }
            div { class: "summary-meta", "LEG {summary.leg_label()} · Ticket {summary.ticket}" }
            ul {
                if summary.leg_deleted {
                    li { "die LEG {summary.leg_label()}" }
                }
                if !summary.participant_ids.is_empty() {
                    li {
                        "{summary.participant_ids.len()} Teilnahme(n)"
                        ul {
                            for id in summary.participant_ids.iter() {
                                li { class: "mono", "{id}" }
                            }
                        }
                    }
                }
                li {
                    "{summary.measuring_points} Messpunkt(e)"
                    if !summary.measuring_point_numbers.is_empty() {
                        ul {
                            for nr in summary.measuring_point_numbers.iter() {
                                li { class: "mono", "{nr}" }
                            }
                        }
                    }
                }
                li { "{summary.contracts} Vertrag/Verträge" }
            }
            if !summary.detached_measuring_point_numbers.is_empty() {
                div { class: "summary-sub",
                    "Bleiben bestehen, nur die Verknüpfung dieser Teilnahme wird entfernt:"
                    ul {
                        for nr in summary.detached_measuring_point_numbers.iter() {
                            li { class: "mono", "{nr}" }
                        }
                    }
                }
            }
            if summary.forced {
                div { class: "summary-sub", "Schutz übersteuert (force)." }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticket_format() {
        assert!(is_valid_ticket("LHSD-1091"));
        assert!(is_valid_ticket("A1-2"));
        assert!(!is_valid_ticket("lhsd-1091"));
        assert!(!is_valid_ticket("1HSD-1091"));
        assert!(!is_valid_ticket("LHSD-"));
        assert!(!is_valid_ticket("LHSD1091"));
        assert!(!is_valid_ticket("LHSD-10-91"));
        assert!(!is_valid_ticket(""));
    }

    #[test]
    fn multi_value_split() {
        assert_eq!(split_multi_value("CH1, CH2;CH3\nCH4  CH5"), vec!["CH1", "CH2", "CH3", "CH4", "CH5"]);
        assert!(split_multi_value(" , ; \n").is_empty());
    }

    #[test]
    fn parses_summary() {
        let body = r#"{"dryRun":true,"forced":false,"ticket":"LHSD-1070","legInternalId":"f79ad9f2",
            "legName":"LEG Stadt Luzern Blattenmoos","legDeleted":false,"participantIds":["e7adabb8"],
            "measuringPointNumbers":["CH1015001234500010060400007001001"],"detachedMeasuringPointNumbers":[],
            "measuringPoints":1,"contracts":0}"#;
        let DeleteOutcome::Summary(s) = interpret_delete_response(200, body) else { panic!("expected summary") };
        assert!(s.dry_run);
        assert_eq!(s.measuring_points, 1);
        assert_eq!(s.leg_label(), "«LEG Stadt Luzern Blattenmoos» (f79ad9f2)");
    }

    #[test]
    fn summary_without_dry_run_flag_is_rejected() {
        assert!(matches!(interpret_delete_response(200, "{}"), DeleteOutcome::Failed { .. }));
    }

    #[test]
    fn conflict_offers_force_unless_manual() {
        let body = r#"{"type":"about:blank","title":"Conflict","status":409,"detail":"LEG is known to the VNB; pass force=true"}"#;
        assert!(matches!(interpret_delete_response(409, body), DeleteOutcome::Failed { can_force: true, .. }));
        let body = r#"{"status":409,"detail":"needs manual clean-up"}"#;
        assert!(matches!(interpret_delete_response(409, body), DeleteOutcome::Failed { can_force: false, .. }));
    }

    #[test]
    fn gateway_timeout_is_warning() {
        let DeleteOutcome::Failed { result, .. } = interpret_delete_response(504, r#"{"status":504,"detail":"x"}"#) else {
            panic!("expected failure")
        };
        assert!(result.kind == ResultKind::Warning);
    }

    #[test]
    fn other_errors_show_detail_or_fallback() {
        let DeleteOutcome::Failed { result, .. } = interpret_delete_response(404, r#"{"detail":"LEG not found"}"#) else {
            panic!()
        };
        assert!(result.message.contains("LEG not found"));
        let DeleteOutcome::Failed { result, .. } = interpret_delete_response(500, "<html>") else { panic!() };
        assert_eq!(result.message, generic_error().message);
    }
}
