# Table Tennis Scoreboard

A wireless, wall-mounted table tennis scoreboard for the office. A 64x64 RGB LED
matrix driven by an ESP32 shows player names, current set/point score, serve
indicator, and past set scores. Controlled entirely over the air — from a
phone (web app, no install) or a small dedicated wireless remote — with no
buttons anywhere near the display itself.

## Status

🚧 Architecture + hardware BOM defined, firmware scaffolded, nothing built or
flashed yet. See [docs/architecture.md](docs/architecture.md) for the plan.

## Repo layout

```
docs/                   Design docs — read these first
  architecture.md       System architecture, network topology, data flow
  hardware-shopping-list.md   Full BOM with approximate costs
  protocol.md            Wire protocol: ESP-NOW controller<->display, WebSocket phone<->display
  match-rules.md         Table tennis scoring/serve rules as implemented

firmware/
  display/               PlatformIO project for the display ESP32
  controller/             PlatformIO project for the dedicated remote's ESP32-C3

hardware/
  README.md               Wiring notes, HUB75 pinout, power/enclosure notes
```

## Quick start (once hardware is in hand)

1. Copy `firmware/display/src/secrets.example.h` to `secrets.h` and fill in
   office WiFi credentials + a shared key for the controller pairing.
2. Open `firmware/display/` and `firmware/controller/` in VS Code with the
   PlatformIO extension (or `pio run` from each directory).
3. Flash the display unit first, note the WiFi channel it lands on (printed
   to serial + shown on an office router status page), then flash the
   controller — see [docs/protocol.md](docs/protocol.md) for why the channel
   matters for ESP-NOW.
4. Visit `http://scoreboard.local` from your phone on the same office WiFi.

## Hardware

See [docs/hardware-shopping-list.md](docs/hardware-shopping-list.md) for the
full BOM. Summary: ESP32 (PSRAM) + 64x64 P3 HUB75 panel + 5V PSU for the
display; ESP32-C3 + arcade buttons + LiPo for the pocket remote.
