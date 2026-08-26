use display_core::{Display, MatchState};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi};

mod display;

// build.rs stages src/secrets.rs (gitignored, real credentials, never
// committed) or falls back to secrets.rs.example into OUT_DIR at build
// time. include!() rather than `mod secrets;` deliberately — see
// build.rs for why a `mod`-based approach doesn't survive `cargo fmt`.
mod secrets {
    include!(concat!(env!("OUT_DIR"), "/secrets.rs"));
}

fn main() {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().expect("peripherals already taken");
    let sys_loop = EspSystemEventLoop::take().expect("system event loop already taken");
    let nvs = EspDefaultNvsPartition::take().expect("NVS partition already taken");

    // The working Arduino reference has `delay(500)` before touching the
    // display at all — giving the panel's power rail time to stabilize
    // after boot. Cheap to keep even though the actual bugs turned out to
    // be elsewhere (SPI3 vs SPI2, and a Rust scope/drop issue below).
    std::thread::sleep(std::time::Duration::from_millis(500));

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))
            .expect("failed to init WiFi driver"),
        sys_loop,
    )
    .expect("failed to wrap WiFi driver");

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: secrets::WIFI_SSID.try_into().expect("WIFI_SSID too long"),
        password: secrets::WIFI_PASSWORD
            .try_into()
            .expect("WIFI_PASSWORD too long"),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    }))
    .expect("failed to set WiFi configuration");

    wifi.start().expect("failed to start WiFi");

    // Checkpoint C: connect once, log clearly either way. No retry/backoff
    // yet — that's a real requirement for the always-on office deployment,
    // not something to guess at now.
    match wifi.connect().and_then(|()| wifi.wait_netif_up()) {
        Ok(()) => {
            let ip_info = wifi
                .wifi()
                .sta_netif()
                .get_ip_info()
                .expect("failed to read IP info");
            log::info!("WiFi connected — IP: {}", ip_info.ip);
        }
        Err(e) => {
            log::error!("WiFi connection failed: {e:?}");
        }
    }

    // Checkpoint D: prove the ST7789 driver works at all. Application
    // isn't wired in yet (Checkpoints E/F), so a hand-built MatchState
    // stands in for real match data.
    match display::St7789Display::new(
        peripherals.spi3,
        peripherals.pins.gpio18,
        peripherals.pins.gpio19,
        peripherals.pins.gpio5,
        peripherals.pins.gpio16,
        peripherals.pins.gpio23,
        peripherals.pins.gpio4,
    ) {
        Ok(mut st7789) => {
            let test_state = MatchState {
                score_left: 7,
                score_right: 5,
                ..MatchState::default()
            };
            st7789.render(&test_state);
            log::info!("St7789Display: rendered test state");

            // st7789 owns the backlight/reset pins and the SPI device —
            // dropping it releases those pins, which is exactly what was
            // causing "renders once, then goes dark" during bring-up: it
            // was previously local to this match arm and got dropped the
            // moment this block ended. Keeping it alive for the program's
            // lifetime by looping right here instead.
            loop {
                std::thread::sleep(std::time::Duration::from_secs(60));
            }
        }
        Err(e) => {
            log::error!("St7789Display::new failed: {e:?}");
            loop {
                std::thread::sleep(std::time::Duration::from_secs(60));
            }
        }
    }
}
