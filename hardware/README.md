# Hardware notes

## HUB75 pinout (default for `ESP32-HUB75-MatrixPanel-I2S-DMA`)

The firmware uses the library's default pin mapping so it works out of the
box with most "ESP32 HUB75 driver shield" boards. If wiring by hand to a bare
dev board, use this mapping (adjust `firmware/display/src/main.cpp` if your
board/shield differs):

| HUB75 signal | ESP32 GPIO |
|---|---|
| R1 | 25 |
| G1 | 26 |
| B1 | 27 |
| R2 | 14 |
| G2 | 12 |
| B2 | 13 |
| A | 23 |
| B | 19 |
| C | 5 |
| D | 17 |
| E | 32 |
| LAT | 4 |
| OE | 15 |
| CLK | 16 |

`E` matters here: a true 64x64 panel is 1/32 scan and needs the `E` address
line wired. Panels/libraries that only need 1/16 scan (e.g. some 64x32
boards) leave it unconnected — don't assume a "64x64" listing is 1/32 scan
without checking, since some are two stacked 1/16 32x64 halves.

## Power

- Feed the panel's 5V directly from the PSU with heavy-enough wire (18AWG
  minimum for a single 64x64 panel) — don't run panel power through the
  ESP32's onboard regulator or USB.
- Common panel ground and ESP32 ground together.
- Add the smoothing capacitor across the panel's power input if it doesn't
  already have one built in.
- Size the PSU for full-white worst case even though typical scoreboard
  content (mostly black background, small colored digits/text) draws much
  less — headroom avoids brownout-induced glitching/resets.

## Controller

- ESP32-C3 + LiPo + TP4056 (protected variant) + slide switch, standard
  "battery-powered ESP32 button" wiring — buttons to GPIO with internal
  pull-ups, debounce in firmware.
- Deep sleep between presses is the main lever for battery life; wake-on-GPIO
  from the two main buttons.

## Not yet decided / revisit once parts are in hand

- Exact shield/board purchased will dictate final pin mapping — update the
  table above once ordered.
- Enclosure design (3D print files will live in this directory once started).
- Whether the panel needs a diffuser for comfortable close-range viewing in
  a small office.
