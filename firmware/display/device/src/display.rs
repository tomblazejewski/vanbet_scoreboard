//! `display_core::Display` implemented against the LILYGO TTGO T-Display's
//! integrated ST7789 panel — the bench display for slice 2. Same trait a
//! future HUB75 driver implements; nothing here is HUB75-specific.
//!
//! Pinout is fixed (soldered on-board, not user-choosable) — the standard
//! values used across the TTGO T-Display community (e.g. TFT_eSPI's
//! Setup25): SCLK=18, MOSI=19, CS=5, DC=16, RST=23, backlight=4.
//!
//! Uses `mipidsi` on **SPI3** (not SPI2 — see git history for the bring-up
//! debugging session that found this, a Rust ownership/drop bug at the
//! call site, a missing DMA config, and an offset gap in an alternate
//! crate that was tried in between). `mipidsi` handles the panel's
//! 135x240-on-a-240x240-controller offset and the portrait->landscape
//! rotation together automatically — pass native (portrait) size/offset,
//! it recomputes the actual address window for whatever orientation is
//! set.

use display_core::{Display, MatchState};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;
use esp_idf_svc::hal::delay::Delay;
use esp_idf_svc::hal::gpio::{Gpio16, Gpio18, Gpio19, Gpio23, Gpio4, Gpio5, Output, PinDriver};
use esp_idf_svc::hal::spi::{config::Config as SpiConfig, Dma, SpiDeviceDriver, SpiDriver, SpiDriverConfig, SPI3};
use esp_idf_svc::hal::units::FromValueType;
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7789;
use mipidsi::options::{ColorInversion, Orientation, Rotation};
use mipidsi::Builder;

type SpiIface<'d> = SpiInterface<'static, SpiDeviceDriver<'d, SpiDriver<'d>>, PinDriver<'d, Output>>;

// mipidsi's SPI interface batches pixel writes through a scratch buffer it
// doesn't own the storage for. Leaked once per display (there's only ever
// one), so `new()` doesn't need a buffer threaded in from the caller.
const DMA_BUFFER_SIZE: usize = 512;

pub struct St7789Display<'d> {
    panel: mipidsi::Display<SpiIface<'d>, ST7789, PinDriver<'d, Output>>,
    // Held for its lifetime, not read again — dropping it would let the
    // pin float and could turn the backlight off.
    _backlight: PinDriver<'d, Output>,
}

impl<'d> St7789Display<'d> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        spi: SPI3<'d>,
        sclk: Gpio18<'d>,
        mosi: Gpio19<'d>,
        cs: Gpio5<'d>,
        dc: Gpio16<'d>,
        rst: Gpio23<'d>,
        backlight: Gpio4<'d>,
    ) -> anyhow::Result<Self> {
        // DMA is off by default and caps transactions at ~64 bytes — a
        // full-screen fill is 64,800 bytes.
        let driver = SpiDriver::new(
            spi,
            sclk,
            mosi,
            None::<esp_idf_svc::hal::gpio::AnyIOPin>,
            &SpiDriverConfig::new().dma(Dma::Auto(4096)),
        )?;

        let spi_device = SpiDeviceDriver::new(
            driver,
            Some(cs),
            &SpiConfig::new().baudrate(20.MHz().into()).data_mode(embedded_hal::spi::MODE_0),
        )?;

        let dc = PinDriver::output(dc)?;
        let rst = PinDriver::output(rst)?;
        let mut backlight = PinDriver::output(backlight)?;
        backlight.set_high()?;

        let buffer: &'static mut [u8; DMA_BUFFER_SIZE] = Box::leak(Box::new([0u8; DMA_BUFFER_SIZE]));
        let di = SpiInterface::new(spi_device, dc, buffer);

        let panel = Builder::new(ST7789, di)
            .display_size(135, 240)
            .display_offset(52, 40)
            .orientation(Orientation::new().rotate(Rotation::Deg270))
            .invert_colors(ColorInversion::Inverted)
            .reset_pin(rst)
            .init(&mut Delay::new_default())
            .map_err(|e| anyhow::anyhow!("ST7789 init failed: {e:?}"))?;

        Ok(Self { panel, _backlight: backlight })
    }
}

impl<'d> Display for St7789Display<'d> {
    fn render(&mut self, state: &MatchState) {
        if self.panel.clear(Rgb565::BLACK).is_err() {
            log::error!("St7789Display: failed to clear panel");
            return;
        }

        let style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
        let text = std::format!("{} - {}", state.score_left, state.score_right);
        if Text::new(&text, Point::new(10, 60), style).draw(&mut self.panel).is_err() {
            log::error!("St7789Display: failed to draw text");
        }
    }
}
