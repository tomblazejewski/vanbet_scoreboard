use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi};

mod secrets;

fn main() {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().expect("peripherals already taken");
    let sys_loop = EspSystemEventLoop::take().expect("system event loop already taken");
    let nvs = EspDefaultNvsPartition::take().expect("NVS partition already taken");

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs)).expect("failed to init WiFi driver"),
        sys_loop,
    )
    .expect("failed to wrap WiFi driver");

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: secrets::WIFI_SSID.try_into().expect("WIFI_SSID too long"),
        password: secrets::WIFI_PASSWORD.try_into().expect("WIFI_PASSWORD too long"),
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
            let ip_info = wifi.wifi().sta_netif().get_ip_info().expect("failed to read IP info");
            log::info!("WiFi connected — IP: {}", ip_info.ip);
        }
        Err(e) => {
            log::error!("WiFi connection failed: {e:?}");
        }
    }

    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}
