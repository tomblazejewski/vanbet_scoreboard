# Controller talks to Display over ESP-NOW, pinned to a fixed router channel

The Controller (physical remote) reaches the Display over ESP-NOW rather than
joining the office WiFi itself, so it avoids the battery cost of a full WiFi
association/keepalive for a device that only fires short point/undo events.
ESP-NOW requires both radios on the same WiFi channel, and the Display's
channel is dictated by whatever the office router assigns when the Display
joins it as a station (needed for phone control via mDNS). We're pinning the
office router to a fixed channel in its own admin settings and hardcoding
that same channel into the Controller's firmware, rather than having the
Controller dynamically discover the Display's current channel at boot.

**Considered and rejected:** the Display hosting its own WiFi AP instead of
joining the office network — would make channel-matching trivial, but breaks
multi-phone access, mDNS, and OTA updates, which we wanted for the phone
control surface.

**Consequence:** if the router's channel ever changes (e.g. auto
channel-select re-picks after interference), the Controller silently stops
reaching the Display until its firmware is re-flashed with the new value —
the router must stay pinned to a fixed channel, not left on auto-select.
