# Hardware shopping list

Budget-sourcing pass (AliExpress/Banggood tier, ~2-4 week shipping). Prices
are ballpark USD from typical current listings, not live quotes — search
these part names/specs directly rather than treating prices as fixed. Say
the word and I can search for specific current listings/sellers.

## Display unit — ~$60–75

| Part | Spec | Approx cost | Notes |
|---|---|---|---|
| LED matrix panel | 64x64 **P4** (4mm pitch) indoor HUB75, 1/32 scan, ~256mm/10in square | $18–25 | "P4 64x64 indoor RGB LED module HUB75" on AliExpress. 1/32 scan means it needs the `E` address line — cheaper 1/16-scan 64x32-style panels don't need it, but a true 64x64 does. |
| ESP32 dev board | Your call — a plain ESP32-WROOM devkit is enough (content here is flat-color text/digits, no PSRAM needed) | $4–8 | You're sourcing/picking this yourself; any HUB75-capable ESP32 dev board works, this row is just a placeholder cost estimate. |
| HUB75 adapter / level shifter | Either a dedicated "ESP32 HUB75 driver shield" that plugs straight onto the dev board, or a bare 74HCT245 breakout wired by hand | $5–8 | The shield is the easier build; bare 74HCT245 is the fallback if a matching shield isn't available for your exact board. |
| Power supply | 5V, 8–10A (40–50W) switching PSU | $8–12 | Size for worst case (full-white frame), even though real usage (scoreboard digits) draws far less on average. |
| HUB75 ribbon cable | 16-pin IDC, usually bundled with the panel | $0–3 | Confirm it's included before ordering separately. |
| Power wiring | 5.5x2.1mm barrel jack + screw terminal block, 18AWG wire | $3–5 | |
| Smoothing capacitor | 1000–2200µF, 6.3V+ electrolytic | $1–2 | Across the panel's 5V input if the panel doesn't already have one on-board (most do — check first). |
| Enclosure/frame | Black acrylic bezel or 3D-printed frame + diffuser | $10–15 | Optional for a first prototype; do this once the electronics are proven. |
| Mounting | Picture-frame hanger or french cleat | $5 | Whatever matches how the office wall is set up. |

## Controller unit — ~$20–30

| Part | Spec | Approx cost | Notes |
|---|---|---|---|
| ESP32-C3 board | e.g. Seeed XIAO ESP32-C3 or a generic ESP32-C3 mini dev board | $4–7 | Small footprint fits a pocket remote; ESP-NOW + deep sleep gives good battery life. |
| Big buttons | 2x arcade-style momentary pushbutton (16–24mm) — Left point, Right point | $3–5/pair | Position-labeled (Left/Right), not player-labeled — see [ADR-0003](adr/0003-side-indexed-by-controller-position.md). |
| Undo button | 1x 6mm tactile button | $0.50 | The Controller has exactly 3 buttons total — no "new set"/spare button, that's out of scope. |
| Battery | LiPo, 500–1000mAh, JST-PH connector | $4–6 | Match the connector to whatever charger board you buy. |
| Charger/protection | TP4056 USB-C module (with protection, not the bare charge-only version) | $1–2 | Get the version with battery protection circuitry built in — bare TP4056 has no over-discharge protection. |
| Power switch | Small SPDT slide switch | $1 | |
| Enclosure | 3D-printed case, or a small project box | $3–8 | |
| Optional: status LED | Single LED (or the C3 board's onboard one) | $0 | For the ESP-NOW ack blink described in `docs/protocol.md`. An OLED is overkill — the Controller never displays names/scores by design, it's buttons-only. |

## Shared / one-time

- USB-C cables for flashing (probably already have these).
- Breadboard + jumper wires for prototyping before committing to a
  soldered build.
- Multimeter, if not already on hand — worth having before powering an
  LED panel for the first time.

## Running total

Roughly **$85–105** for a first working display + remote, before enclosure
polish. The panel and PSU are the parts most worth NOT cutting corners on
(DOA panels are a pain to return from overseas sellers) — worth reading
recent buyer reviews/photos on the specific listing before ordering, even
within the budget tier.
