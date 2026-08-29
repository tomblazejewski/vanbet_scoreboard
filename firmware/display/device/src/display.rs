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

use crate::clock::Clock;
use display_core::{Display, MatchState, Side};
use display_render::{build_view, ScoreboardView};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::mono_font::ascii::{FONT_10X20, FONT_6X10};
use embedded_graphics::mono_font::{MonoFont, MonoTextStyle};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;
use esp_idf_svc::hal::delay::Delay;
use esp_idf_svc::hal::gpio::{Gpio16, Gpio18, Gpio19, Gpio23, Gpio4, Gpio5, Output, PinDriver};
use esp_idf_svc::hal::spi::{
    config::Config as SpiConfig, Dma, SpiDeviceDriver, SpiDriver, SpiDriverConfig, SPI3,
};
use esp_idf_svc::hal::units::FromValueType;
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7789;
use mipidsi::options::{ColorInversion, Orientation, Rotation};
use mipidsi::Builder;

type SpiIface<'d> =
    SpiInterface<'static, SpiDeviceDriver<'d, SpiDriver<'d>>, PinDriver<'d, Output>>;

// mipidsi's SPI interface batches pixel writes through a scratch buffer it
// doesn't own the storage for. Leaked once per display (there's only ever
// one), so `new()` doesn't need a buffer threaded in from the caller.
const DMA_BUFFER_SIZE: usize = 512;

// This screen's own space budget for display_render::build_view — the
// right-hand column (see draw_match) is only about 95px wide, so both
// numbers are noticeably smaller than the screen's full character width
// would allow. Names get their own full-width line each rather than a
// single "left vs right" line (see draw_match) — two lines at up to 10
// chars fits the column; one combined line wouldn't. Two history entries,
// not more, matches architecture.md's own HUB75 reference sketch, which
// shows exactly two past Sets plus an ellipsis for anything older.
const MAX_NAME_CHARS: usize = 10;
const MAX_HISTORY_ENTRIES: usize = 2;

// Right column's left edge — everything left of this is the big score;
// everything at or past it is names/sets/history/decided.
const COLUMN_X: i32 = 148;

pub struct St7789Display<'d> {
    panel: mipidsi::Display<SpiIface<'d>, ST7789, PinDriver<'d, Output>>,
    // Held for its lifetime, not read again — dropping it would let the
    // pin float and could turn the backlight off.
    _backlight: PinDriver<'d, Output>,
    clock: Clock,
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
        clock: Clock,
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
            &SpiConfig::new()
                .baudrate(20.MHz().into())
                .data_mode(embedded_hal::spi::MODE_0),
        )?;

        let dc = PinDriver::output(dc)?;
        let rst = PinDriver::output(rst)?;
        let mut backlight = PinDriver::output(backlight)?;
        backlight.set_high()?;

        let buffer: &'static mut [u8; DMA_BUFFER_SIZE] =
            Box::leak(Box::new([0u8; DMA_BUFFER_SIZE]));
        let di = SpiInterface::new(spi_device, dc, buffer);

        let panel = Builder::new(ST7789, di)
            .display_size(135, 240)
            .display_offset(52, 40)
            .orientation(Orientation::new().rotate(Rotation::Deg270))
            .invert_colors(ColorInversion::Inverted)
            .reset_pin(rst)
            .init(&mut Delay::new_default())
            .map_err(|e| anyhow::anyhow!("ST7789 init failed: {e:?}"))?;

        Ok(Self {
            panel,
            _backlight: backlight,
            clock,
        })
    }

    fn draw_line(&mut self, text: &str, pos: Point, font: &MonoFont, color: Rgb565) {
        let style = MonoTextStyle::new(font, color);
        if Text::new(text, pos, style).draw(&mut self.panel).is_err() {
            log::error!("St7789Display: failed to draw text");
        }
    }

    /// The full content model from `display_render`, laid out for this
    /// screen specifically — see `MAX_NAME_CHARS`/`MAX_HISTORY_ENTRIES`/
    /// `COLUMN_X` for this display's own space budget and grid. Big score
    /// on the left (the single most important fact), names/server/sets/
    /// history/decided stacked in a narrower column on the right — this
    /// screen is landscape (240x135, wider than it is tall), so a
    /// two-column layout uses the shape better than stacking everything
    /// in one column would. Same facts `docs/architecture.md`'s Rendering
    /// section describes for the eventual (square, much lower-resolution)
    /// HUB75 panel; that panel will need its own, more cramped layout —
    /// this one is specific to this screen's shape, not a template to
    /// reuse verbatim.
    fn draw_match(&mut self, view: &ScoreboardView) {
        let score = std::format!("{}-{}", view.score_left, view.score_right);
        self.draw_line(&score, Point::new(6, 80), &FONT_10X20, Rgb565::WHITE);

        let marker = |side: Side| if view.server == side { "*" } else { " " };
        self.draw_line(
            &std::format!("{}{}", marker(Side::Left), view.left_name),
            Point::new(COLUMN_X, 24),
            &FONT_6X10,
            Rgb565::WHITE,
        );
        self.draw_line(
            &std::format!("{}{}", marker(Side::Right), view.right_name),
            Point::new(COLUMN_X, 36),
            &FONT_6X10,
            Rgb565::WHITE,
        );

        let sets = std::format!("Sets: {}-{}", view.sets_won_left, view.sets_won_right);
        self.draw_line(&sets, Point::new(COLUMN_X, 54), &FONT_6X10, Rgb565::WHITE);

        if !view.history.is_empty() {
            let entries: Vec<String> = view
                .history
                .iter()
                .map(|(l, r)| std::format!("{l}-{r}"))
                .collect();
            let prefix = if view.history_truncated { "..." } else { "" };
            let text = std::format!("{prefix}{}", entries.join(" "));
            self.draw_line(&text, Point::new(COLUMN_X, 66), &FONT_6X10, Rgb565::WHITE);
        }

        if view.decided {
            self.draw_line(
                "DECIDED",
                Point::new(COLUMN_X, 90),
                &FONT_6X10,
                Rgb565::YELLOW,
            );
        }
    }
}

impl<'d> Display for St7789Display<'d> {
    fn render(&mut self, state: &MatchState) {
        if self.panel.clear(Rgb565::BLACK).is_err() {
            log::error!("St7789Display: failed to clear panel");
            return;
        }

        // Persistent regardless of Match state — see
        // docs/slices/02-display-bringup-plan.md's Checkpoint H.
        let time = self.clock.now_hh_mm();
        self.draw_line(&time, Point::new(190, 6), &FONT_6X10, Rgb565::WHITE);

        match build_view(state, MAX_NAME_CHARS, MAX_HISTORY_ENTRIES) {
            Some(view) => self.draw_match(&view),
            None => self.draw_line("No Match", Point::new(6, 75), &FONT_10X20, Rgb565::WHITE),
        }
    }
}
