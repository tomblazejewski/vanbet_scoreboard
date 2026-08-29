use display_core::Application;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi};
use std::sync::{Arc, Mutex};

mod clock;
mod display;
mod rest_server;
mod storage;

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

    // Checkpoint H: SNTP needs WiFi already up — started regardless of
    // whether the connect attempt above actually succeeded, since it just
    // retries quietly in the background either way; now_hh_mm() reports
    // "--:--" until it manages to sync.
    let clock = clock::Clock::start().expect("Clock::start failed");

    let st7789 = display::St7789Display::new(
        peripherals.spi3,
        peripherals.pins.gpio18,
        peripherals.pins.gpio19,
        peripherals.pins.gpio5,
        peripherals.pins.gpio16,
        peripherals.pins.gpio23,
        peripherals.pins.gpio4,
        clock,
    )
    .expect("St7789Display::new failed");

    // Checkpoint E: real Application, driven by the REST layer instead of
    // a hand-built MatchState. new() renders the resumed (here: default
    // Standby, since NoopStorage never has anything to resume) state
    // immediately.
    let app = Arc::new(Mutex::new(Application::new(st7789, storage::NoopStorage)));

    // Keeping the returned EspHttpServer alive for the program's lifetime
    // matters exactly the way it did for St7789Display in Checkpoint D —
    // dropping it tears the server down.
    let _server = rest_server::start(app.clone()).expect("rest_server::start failed");

    // Checkpoint H: the clock shown on the display needs to keep ticking
    // even when nobody's tapping anything — handle() only re-renders in
    // response to a Command, so without this the clock would freeze at
    // whatever it showed during the last scoring action. refresh()
    // re-renders the current (unchanged) state on a timer instead.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
        app.lock().expect("Application mutex poisoned").refresh();
    }
}
