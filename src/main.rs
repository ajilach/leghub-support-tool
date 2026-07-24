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
    // Vorübergehend deaktiviert: Endpunkt noch nicht erreichbar.
    /*
    Action {
        group: "Gründungen",
        title: "Gründung anpassen",
        description: "Ändert einzelne Angaben einer bestehenden Gründung. Nur die angegebenen Felder werden überschrieben, alles andere bleibt unverändert.",
        path_suffix: "patchFounding",
        fields: &[
            Field { key: "legId", label: "Gründungs-ID", hint: "ID wie im Dashboard aufgelistet", required: true },
        ],
    },
    */
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
    // Vorübergehend deaktiviert: Endpunkt noch nicht erreichbar.
    /*
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
    */
    Action {
        group: "Unterschriften",
        title: "Unterschrift als abgeschlossen markieren",
        description: "Behandelt eine Unterschrift so, als wäre sie fertig unterzeichnet worden.",
        path_suffix: "signatureProcessCompleted",
        fields: &[Field { key: "documentId", label: "Dokument-ID", hint: "ID wie im SC Sign Mail angezeigt", required: true }],
    },
];

/// Reihenfolge der Gruppen im Seitenpanel.
static GROUPS: &[&str] = &["Gründungen", "Teilnahmen", "Unterschriften"];

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

static PATCH_FIELDS_FOUNDING: &[PatchField] = &[
    PatchField { path: "identity.name", label: "Name der Gründung", kind: ValueKind::Text },
    PatchField { path: "identity.externalLegId", label: "Externe Gründungs-ID", kind: ValueKind::Text },
    PatchField { path: "visibility.legVisibility", label: "Sichtbarkeit", kind: ValueKind::Choice(VISIBILITY) },
    PatchField { path: "visibility.listFullAddress", label: "Vollständige Adresse öffentlich anzeigen", kind: ValueKind::Bool },
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
    PatchField { path: "billingConfiguration.billingType", label: "Abrechnungsart", kind: ValueKind::Choice(BILLING_TYPE) },
    PatchField { path: "billingConfiguration.pricingModel", label: "Preismodell", kind: ValueKind::Choice(PRICING_MODEL) },
    PatchField { path: "billingConfiguration.individualFixedPrice", label: "Individueller Fixpreis", kind: ValueKind::Number },
    PatchField { path: "billingConfiguration.standardTariffDiscount", label: "Rabatt Standardtarif", kind: ValueKind::Number },
    PatchField { path: "billingConfiguration.bankAccount.iban", label: "IBAN", kind: ValueKind::Text },
    PatchField { path: "billingConfiguration.bankAccount.accountHolder", label: "Kontoinhaber", kind: ValueKind::Text },
    PatchField { path: "participation.allowedParticipants", label: "Erlaubte Teilnehmer", kind: ValueKind::Choice(PARTICIPANT_TYPE) },
    PatchField { path: "ownership.founder.contact.firstName", label: "Gründer: Vorname", kind: ValueKind::Text },
    PatchField { path: "ownership.founder.contact.lastName", label: "Gründer: Nachname", kind: ValueKind::Text },
    PatchField { path: "ownership.founder.contact.email", label: "Gründer: E-Mail", kind: ValueKind::Text },
    PatchField { path: "ownership.founder.contact.phone", label: "Gründer: Telefon", kind: ValueKind::Text },
    PatchField { path: "ownership.founder.contact.companyName", label: "Gründer: Firma", kind: ValueKind::Text },
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

    let action = &ACTIONS[selected()];
    let patch_fields = patch_fields_for(action.path_suffix);

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
                    for group in GROUPS.iter() {
                        div { class: "nav-group", "{group}" }
                        for (i, a) in ACTIONS.iter().enumerate() {
                            if a.group == *group {
                                button {
                                    class: if selected() == i { "nav-item active" } else { "nav-item" },
                                    onclick: move |_| {
                                        selected.set(i);
                                        values.write().clear();
                                        patch_changes.write().clear();
                                        result.set(None);
                                    },
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
            main { class: "content",
                div { class: "env-badge",
                    "Umgebung: "
                    span { class: if settings.read().env == Env::Prod { "env-prod" } else { "env-test" },
                        "{settings.read().env.label()}"
                    }
                }
                h1 { "{action.title}" }
                p { class: "description", "{action.description}" }

                form {
                    onsubmit: move |e| e.stop_propagation(),
                    for field in action.fields.iter() {
                        div { class: "field",
                            label {
                                "{field.label}"
                                if !field.required {
                                    span { class: "optional", " (optional)" }
                                }
                            }
                            input {
                                r#type: "text",
                                autocomplete: "off",
                                value: "{values.read().get(field.key).cloned().unwrap_or_default()}",
                                oninput: move |e| {
                                    values.write().insert(field.key.to_string(), e.value());
                                },
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
                            div { class: "hint", "Wählen Sie unten die Felder aus, die geändert werden sollen." }

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
                                                                if let Some(c) = patch_changes.write().get_mut(idx) { c.1 = e.value(); }
                                                            },
                                                            option { value: "true", "Ja" }
                                                            option { value: "false", "Nein" }
                                                        }
                                                    },
                                                    ValueKind::Choice(opts) => rsx! {
                                                        select {
                                                            value: "{change.1}",
                                                            onchange: move |e| {
                                                                if let Some(c) = patch_changes.write().get_mut(idx) { c.1 = e.value(); }
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
                                                                if let Some(c) = patch_changes.write().get_mut(idx) { c.1 = e.value(); }
                                                            },
                                                        }
                                                    },
                                                    ValueKind::Date | ValueKind::LocalDate => rsx! {
                                                        input {
                                                            r#type: "date",
                                                            value: "{change.1}",
                                                            oninput: move |e| {
                                                                if let Some(c) = patch_changes.write().get_mut(idx) { c.1 = e.value(); }
                                                            },
                                                        }
                                                    },
                                                    ValueKind::Text => rsx! {
                                                        input {
                                                            r#type: "text",
                                                            autocomplete: "off",
                                                            value: "{change.1}",
                                                            oninput: move |e| {
                                                                if let Some(c) = patch_changes.write().get_mut(idx) { c.1 = e.value(); }
                                                            },
                                                        }
                                                    },
                                                }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "patch-remove",
                                                title: "Entfernen",
                                                onclick: move |_| { patch_changes.write().remove(idx); },
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
                                    if path.is_empty() { return; }
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

                    button {
                        r#type: "button",
                        class: "run-btn",
                        disabled: sending(),
                        onclick: move |_| {
                            let action = &ACTIONS[selected()];
                            let current = settings.read().clone();

                            // Zugangsschlüssel für die gewählte Umgebung vorhanden?
                            if current.key().trim().is_empty() {
                                result.set(Some(ActionResult {
                                    kind: ResultKind::Error,
                                    message: format!(
                                        "Für die Umgebung \"{}\" ist kein Zugangsschlüssel hinterlegt. Bitte in den Einstellungen eintragen.",
                                        current.env.label()
                                    ),
                                }));
                                show_settings.set(true);
                                return;
                            }

                            // Pflichtfelder (IDs usw.) prüfen & sammeln.
                            let is_patch = !patch_fields.is_empty();
                            let vals = values.read();
                            let mut query: Vec<(String, String)> = Vec::new();
                            let mut form: Vec<(String, String)> = Vec::new();
                            for f in action.fields.iter() {
                                let v = vals.get(f.key).map(|s| s.trim().to_string()).unwrap_or_default();
                                if f.required && v.is_empty() {
                                    result.set(Some(ActionResult {
                                        kind: ResultKind::Error,
                                        message: "Bitte alle Pflichtfelder ausfüllen.".into(),
                                    }));
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

                            // Bei "anpassen"-Aktionen die ausgewählten Änderungen zu JSON zusammensetzen.
                            let body = if is_patch {
                                let changes = patch_changes.read();
                                if changes.is_empty() {
                                    result.set(Some(ActionResult {
                                        kind: ResultKind::Error,
                                        message: "Bitte mindestens ein zu änderndes Feld auswählen.".into(),
                                    }));
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
                                                result.set(Some(ActionResult {
                                                    kind: ResultKind::Error,
                                                    message: format!("Bei \"{}\" bitte eine gültige Zahl eingeben.", pf.label),
                                                }));
                                                return;
                                            }
                                        }
                                        ValueKind::Date => {
                                            if value.is_empty() {
                                                result.set(Some(ActionResult {
                                                    kind: ResultKind::Error,
                                                    message: format!("Bei \"{}\" bitte ein Datum wählen.", pf.label),
                                                }));
                                                return;
                                            }
                                            serde_json::Value::String(format!("{value}T00:00:00Z"))
                                        }
                                        _ => {
                                            if value.is_empty() {
                                                result.set(Some(ActionResult {
                                                    kind: ResultKind::Error,
                                                    message: format!("Bei \"{}\" bitte einen Wert angeben.", pf.label),
                                                }));
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
                                current.env.host(),
                                action.path_suffix
                            );
                            let key = current.key().trim().to_string();

                            // Nicht sofort senden: erst im Popup bestätigen lassen.
                            result.set(None);
                            confirm_checked.set(false);
                            pending.set(Some(PendingRequest {
                                url,
                                key,
                                query,
                                body,
                                action_title: action.title.to_string(),
                                env: current.env,
                            }));
                        },
                        if sending() { "Wird ausgeführt …" } else { "Ausführen" }
                    }
                }

                if let Some(res) = result() {
                    div {
                        class: match res.kind {
                            ResultKind::Success => "result result-ok",
                            ResultKind::Error => "result result-err",
                        },
                        "{res.message}"
                    }
                }
            }

            // -------- Einstellungen (Overlay) --------
            if show_settings() {
                div { class: "overlay", onclick: move |_| show_settings.set(false),
                    div { class: "modal", onclick: move |e| e.stop_propagation(),
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
                                    },
                                    "Test"
                                }
                                button {
                                    r#type: "button",
                                    class: if settings.read().env == Env::Prod { "env-opt active env-opt-prod" } else { "env-opt" },
                                    onclick: move |_| {
                                        settings.write().env = Env::Prod;
                                        save_settings(&settings.read());
                                    },
                                    "Produktion"
                                }
                            }
                        }

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
                    div { class: "modal", onclick: move |e| e.stop_propagation(),
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

                        label { class: "confirm-check",
                            input {
                                r#type: "checkbox",
                                checked: confirm_checked(),
                                onchange: move |e| confirm_checked.set(e.checked()),
                            }
                            span { "Ja, ich möchte diese Aktion jetzt ausführen." }
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
                                class: "run-btn",
                                disabled: !confirm_checked(),
                                onclick: move |_| {
                                    if let Some(req) = pending() {
                                        pending.set(None);
                                        confirm_checked.set(false);
                                        sending.set(true);
                                        result.set(None);
                                        spawn(async move {
                                            let outcome = send_request(req.url, req.key, req.query, req.body).await;
                                            result.set(Some(outcome));
                                            sending.set(false);
                                        });
                                    }
                                },
                                "Ausführen"
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
    let mut builder = client.post(&url).header("X-Api-Key", key);
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
                ActionResult {
                    kind: ResultKind::Error,
                    message: "Der Zugangsschlüssel fehlt oder ist falsch. Bitte in den Einstellungen prüfen.".into(),
                }
            } else {
                ActionResult {
                    kind: ResultKind::Error,
                    message: "Die Aktion konnte nicht ausgeführt werden. Bitte die eingegebene ID prüfen und erneut versuchen.".into(),
                }
            }
        }
        Err(_) => ActionResult {
            kind: ResultKind::Error,
            message: "Der Server ist nicht erreichbar. Bitte Internetverbindung und Umgebung prüfen.".into(),
        },
    }
}
