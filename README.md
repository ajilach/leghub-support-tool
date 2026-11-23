# LegHub Support-Tool

Ein kleines Programm, mit dem Support-Aufgaben für LegHub-Gründungen und
-Teilnahmen mit wenigen Klicks ausgelöst werden können – ohne technische Hilfsmittel.

## So verwendest du das Tool

1. Programm starten. Es öffnet sich die **Übersicht** mit den beiden Abläufen
   (Gründung und Teilnahme). Sie zeigt die Stationen, an denen ein Vorgang hängen
   bleiben kann – ein Klick auf eine Aktion öffnet direkt das passende Formular.
   Links erscheint zusätzlich die Liste aller Aktionen.
2. Unten links auf **⚙ Einstellungen** klicken und einmalig:
   - die **Umgebung** wählen (**Test** zum Ausprobieren, **Produktion** für den Echtbetrieb),
   - den **Zugangsschlüssel** eintragen (erhältst du von der IT).
   Beides wird gemerkt und muss nicht jedes Mal neu eingegeben werden.
3. In der Übersicht auf die passende Station klicken – oder links die gewünschte
   **Aktion** auswählen.
4. Die verlangten Felder (z. B. Gründungs-ID) ausfüllen und auf **Ausführen** klicken.
5. Es erscheint eine grüne Bestätigung, wenn alles geklappt hat, sonst ein roter Hinweis.

> **Achtung:** In der Umgebung **Produktion** wirken die Aktionen sofort auf echte Daten.

### Austritt & Auflösung

**LEG auflösen** und **Messpunkte austreten lassen** lösen denselben Ablauf aus wie das Dashboard
(Meldung an den Netzbetreiber bzw. innosolv, danach Enddatum in LEGhub), aber mit **frei wählbarem Datum**.

- Es gibt **keinen Probelauf**: Das Enddatum ist danach unveränderlich. Das Bestätigungs-Popup zeigt LEG,
  Messpunkte und Datum noch einmal und warnt bei einem Datum in der Vergangenheit oder weit in der Zukunft.
- Abgelehnt wird nur ein Datum vor dem Start, ein bereits gesetztes Enddatum oder was der Netzbetreiber ablehnt.
  Das Ergebnis zeigt pro Messpunkt, was erledigt ist und was nicht, mit Fehlerquelle (Prüfung in LEGhub,
  Netzbetreiber, LEGhub-Datenbank).
- Bei einem wiederholbaren Fehler (z. B. Speichern fehlgeschlagen) oder ausgebliebener Antwort erscheint
  **Erneut versuchen**. Das ist unschädlich; beim Messpunkt-Austritt werden nur die noch offenen Messpunkte
  wiederholt.
- **Nur in LEGhub beenden** überspringt die Meldung an den Netzbetreiber – für Netzbetreiber, deren
  Schnittstelle noch keine Austritte kann. Der Netzbetreiber muss dann separat informiert werden.

### Löschen

**LEG löschen**, **Teilnahme löschen** und **Messpunkte löschen** laufen immer in zwei Schritten:

1. **Probelauf starten**: Es wird nichts gelöscht. Das Tool zeigt an, was gelöscht würde.
2. Erst danach erscheint **Jetzt wirklich löschen**. Gelöscht wird genau das, was im Probelauf geprüft
   wurde. Wird danach ein Feld geändert, muss erneut ein Probelauf gemacht werden.

Meldet der Probelauf eine Schutzregel, weil der Netzbetreiber die Daten schon kennt, erscheint die Option
**Schutz übersteuern**. Wann sie angebracht ist, steht direkt in der App. Hintergrund:
[Confluence – Support-Endpoints: LEG, Teilnahme und Messpunkte löschen](https://ajila.atlassian.net/wiki/spaces/SWPWRLEG/pages/5669978123).

## Für Entwickler:innen

Voraussetzung: Rust und die Dioxus-CLI (`dx`).

```bash
# Entwicklung
dx serve --platform desktop

# oder direkt
cargo run

# Fertiges Programm bauen
dx bundle --platform desktop
```

Für lokale Tests lässt sich der Server in Debug-Builds überschreiben (Release-Builds ignorieren das):

```bash
LEGHUB_SUPPORT_HOST=http://localhost:4502 dx serve --platform desktop
```

Die Einstellungen werden lokal gespeichert unter dem Konfigurationsverzeichnis von
`LegHub Support-Tool` (macOS: `~/Library/Application Support/`).

Das Ablauf-Diagramm der Übersicht steht als statische Daten in `src/flows.rs`. Es ist
eine bewusst auf die Support-Stationen reduzierte Fassung des vollständigen
Prozessdiagramms unter
<https://ajilach.github.io/leghub-openapi/process/LEGhub_User_Flows_aktuell.html> und
muss von Hand nachgezogen werden, wenn sich der Ablauf ändert.
