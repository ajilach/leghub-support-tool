//! Übersichtsseite: der LegHub-Ablauf, reduziert auf die Stationen, an denen der
//! Support eingreifen kann.
//!
//! Das ist bewusst **keine** Kopie des vollständigen Prozessdiagramms
//! (<https://ajilach.github.io/leghub-openapi/process/LEGhub_User_Flows_aktuell.html>,
//! Stand 2026-06-12). E-Mails, v1/v2-Validierung, Test-Shortcuts und
//! Entscheidungsverzweigungen sind weggelassen — übrig bleiben die Punkte, an denen ein
//! Vorgang steckenbleiben kann, mit den jeweils passenden Aktionen.
//!
//! Ändert sich der Ablauf, muss diese Datei von Hand nachgezogen werden.

use dioxus::prelude::*;

use crate::ACTIONS;

/// Bestimmt nur die Farbgebung der Karte.
#[derive(Clone, Copy, PartialEq)]
enum StageKind {
    /// Vorgang ist erfasst, der Ablauf hat noch nicht (oder nicht mehr) gegriffen.
    Draft,
    /// Wartet auf die Freigabe einer Person.
    Approval,
    /// Wartet auf die elektronische Unterschrift.
    Signature,
    /// LEGhub speichert in den Fremdsystemen.
    Persist,
    /// Endzustand.
    Done,
}

impl StageKind {
    fn class(self) -> &'static str {
        match self {
            StageKind::Draft => "flow-stage kind-draft",
            StageKind::Approval => "flow-stage kind-approval",
            StageKind::Signature => "flow-stage kind-signature",
            StageKind::Persist => "flow-stage kind-persist",
            StageKind::Done => "flow-stage kind-done",
        }
    }
}

struct Stage {
    /// Zustand, wie er im Dashboard angezeigt wird (leer = kein eigener Zustand).
    state: &'static str,
    title: &'static str,
    /// Wer an dieser Stelle am Zug ist.
    actor: &'static str,
    /// Woran man erkennt, dass ein Vorgang hier hängt (leer = kein Hinweis).
    stuck: &'static str,
    /// Wann diese Station ganz entfällt (leer = sie kommt immer vor).
    skipped: &'static str,
    /// Auslösbare Aktionen: (path_suffix aus ACTIONS, Kurzbeschriftung).
    actions: &'static [(&'static str, &'static str)],
    kind: StageKind,
}

struct Flow {
    title: &'static str,
    subtitle: &'static str,
    stages: &'static [Stage],
}

static FLOWS: &[Flow] = &[
    Flow {
        title: "Gründung",
        subtitle: "Eine Person gründet eine neue LEG",
        stages: &[
            Stage {
                state: "DRAFT",
                title: "Gründung angelegt",
                actor: "LEGhub",
                stuck: "die Prüfanfrage an den Netzbetreiber nie versendet wurde – oder das Speichern fehlgeschlagen ist und der Vorgang zurückgesetzt wurde.",
                skipped: "",
                actions: &[
                    ("patchFounding", "Angaben anpassen"),
                    ("restartFounding", "Neu starten"),
                ],
                kind: StageKind::Draft,
            },
            Stage {
                state: "WAITING_FOR_VNB_APPROVAL",
                title: "Freigabe durch den Netzbetreiber",
                actor: "Netzbetreiber",
                stuck: "der Netzbetreiber nicht reagiert oder das Prüf-Mail nicht erhalten hat.",
                skipped: "wenn die automatische Freigabe durch den Netzbetreiber aktiviert ist.",
                actions: &[
                    ("founding/acceptVnb", "Annehmen"),
                    ("founding/rejectVnb", "Ablehnen"),
                ],
                kind: StageKind::Approval,
            },
            Stage {
                state: "WAITING_FOR_SIGNATURE",
                title: "Unterschrift (Swisscom Sign)",
                actor: "Gründer",
                stuck: "die Einladung nicht angekommen ist – oder unterschrieben wurde, die Rückmeldung von Swisscom Sign aber ausgeblieben ist.",
                skipped: "wenn kein Produzent beteiligt ist (leere Hülle).",
                actions: &[
                    ("retriggerFoundingSignature", "Anfrage erneut senden"),
                    ("signatureProcessCompleted", "Als unterschrieben markieren"),
                ],
                kind: StageKind::Signature,
            },
            Stage {
                state: "",
                title: "Speichern im System des Netzbetreibers und in LEGhub",
                actor: "LEGhub",
                stuck: "das Speichern fehlgeschlagen ist: der Vorgang steht wieder auf DRAFT und der Netzbetreiber hat ein Fehler-Mail erhalten.",
                skipped: "",
                actions: &[("founding/retryVnb", "Speichern erneut versuchen")],
                kind: StageKind::Persist,
            },
            Stage {
                state: "WAITING_FOR_PRODUCER / WAITING_FOR_CONSUMER",
                title: "Gründung bestätigt – die LEG wartet auf Teilnehmende",
                actor: "Abgeschlossen",
                stuck: "",
                skipped: "",
                actions: &[],
                kind: StageKind::Done,
            },
        ],
    },
    Flow {
        title: "Teilnahme",
        subtitle: "Eine Person tritt einer bestehenden LEG bei",
        stages: &[
            Stage {
                state: "DRAFT",
                title: "Beitritt erfasst",
                actor: "LEGhub",
                stuck: "der Beitritt nach einer Ablehnung oder einem Speicherfehler zurückgesetzt wurde.",
                skipped: "",
                actions: &[
                    ("patchParticipation", "Angaben anpassen"),
                    ("restartParticipation", "Neu starten"),
                ],
                kind: StageKind::Draft,
            },
            Stage {
                state: "WAITING_FOR_FOUNDER_APPROVAL",
                title: "Freigabe durch Gründer / LEG-Vertretung",
                actor: "Gründer",
                stuck: "die Gründerin oder der Gründer nicht reagiert oder das Prüf-Mail nicht erhalten hat.",
                skipped: "",
                actions: &[
                    ("participation/acceptFounder", "Annehmen"),
                    ("participation/rejectFounder", "Ablehnen"),
                ],
                kind: StageKind::Approval,
            },
            Stage {
                state: "WAITING_FOR_VNB_APPROVAL",
                title: "Freigabe durch den Netzbetreiber",
                actor: "Netzbetreiber",
                stuck: "der Netzbetreiber nicht reagiert oder das Prüf-Mail nicht erhalten hat.",
                skipped: "wenn die automatische Freigabe durch den Netzbetreiber aktiviert ist.",
                actions: &[
                    ("participation/acceptVnb", "Annehmen"),
                    ("participation/rejectVnb", "Ablehnen"),
                ],
                kind: StageKind::Approval,
            },
            Stage {
                state: "WAITING_FOR_SIGNATURE",
                title: "Unterschrift (Swisscom Sign)",
                actor: "Teilnehmer",
                stuck: "die Einladung nicht angekommen ist – oder unterschrieben wurde, die Rückmeldung von Swisscom Sign aber ausgeblieben ist.",
                skipped: "wenn für die LEG kein Unterschrifts-Prozess vorgesehen ist.",
                actions: &[
                    ("retriggerParticipationSignature", "Anfrage erneut senden"),
                    ("signatureProcessCompleted", "Als unterschrieben markieren"),
                ],
                kind: StageKind::Signature,
            },
            Stage {
                state: "",
                title: "Speichern im System des Netzbetreibers und in LEGhub",
                actor: "LEGhub",
                stuck: "das Speichern fehlgeschlagen ist: der Beitritt steht wieder auf DRAFT und der Netzbetreiber hat ein Fehler-Mail erhalten.",
                skipped: "",
                actions: &[("participation/retryVnb", "Speichern erneut versuchen")],
                kind: StageKind::Persist,
            },
            Stage {
                state: "ACTIVE",
                title: "Beitritt bestätigt – die LEG wird aktiv, sobald sie vollständig ist",
                actor: "Abgeschlossen",
                stuck: "",
                skipped: "",
                actions: &[],
                kind: StageKind::Done,
            },
        ],
    },
];

/// Vollständiger Aktionsname für den Tooltip eines Chips.
fn action_title(path_suffix: &str) -> &'static str {
    ACTIONS
        .iter()
        .find(|a| a.path_suffix == path_suffix)
        .map(|a| a.title)
        .unwrap_or("")
}

/// Übersichtsseite: beide Abläufe nebeneinander, Aktionen direkt anklickbar.
#[component]
pub fn Overview(on_open: EventHandler<&'static str>) -> Element {
    rsx! {
        div { class: "overview",
            h1 { "Übersicht" }
            p { class: "description",
                "Wo hängt der Vorgang? Wählen Sie unten die passende Station – ein Klick auf "
                "eine Aktion öffnet das zugehörige Formular. Der Zustand eines Vorgangs ist im "
                "Dashboard ersichtlich."
            }

            div { class: "flow-grid",
                for flow in FLOWS.iter() {
                    div { class: "flow",
                        div { class: "flow-head",
                            h2 { "{flow.title}" }
                            div { class: "flow-sub", "{flow.subtitle}" }
                        }

                        for (i, stage) in flow.stages.iter().enumerate() {
                            if i > 0 {
                                div { class: "flow-connector" }
                            }
                            div { class: stage.kind.class(),
                                div { class: "flow-stage-head",
                                    span { class: "flow-actor", "{stage.actor}" }
                                    if !stage.state.is_empty() {
                                        span { class: "flow-state", "{stage.state}" }
                                    }
                                }
                                div { class: "flow-stage-title", "{stage.title}" }
                                if !stage.stuck.is_empty() {
                                    div { class: "flow-stuck", "Hängt hier, wenn {stage.stuck}" }
                                }
                                if !stage.skipped.is_empty() {
                                    div { class: "flow-skip", "Entfällt, {stage.skipped}" }
                                }
                                if !stage.actions.is_empty() {
                                    div { class: "flow-chips",
                                        for (suffix, label) in stage.actions.iter() {
                                            button {
                                                r#type: "button",
                                                class: "flow-chip",
                                                title: "{action_title(suffix)}",
                                                onclick: move |_| on_open.call(suffix),
                                                "{label}"
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
    }
}
