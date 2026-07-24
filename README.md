# LegHub Support-Tool

Ein kleines Programm, mit dem Support-Aufgaben für LegHub-Gründungen und
-Teilnahmen mit wenigen Klicks ausgelöst werden können – ohne technische Hilfsmittel.

## So verwendest du das Tool

1. Programm starten. Links erscheint eine Liste der möglichen Aktionen.
2. Unten links auf **⚙ Einstellungen** klicken und einmalig:
   - die **Umgebung** wählen (**Test** zum Ausprobieren, **Produktion** für den Echtbetrieb),
   - den **Zugangsschlüssel** eintragen (erhältst du von der IT).
   Beides wird gemerkt und muss nicht jedes Mal neu eingegeben werden.
3. Links die gewünschte **Aktion** auswählen.
4. Die verlangten Felder (z. B. Gründungs-ID) ausfüllen und auf **Ausführen** klicken.
5. Es erscheint eine grüne Bestätigung, wenn alles geklappt hat, sonst ein roter Hinweis.

> **Achtung:** In der Umgebung **Produktion** wirken die Aktionen sofort auf echte Daten.

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

Die Einstellungen werden lokal gespeichert unter dem Konfigurationsverzeichnis von
`LegHub Support-Tool` (macOS: `~/Library/Application Support/`).
