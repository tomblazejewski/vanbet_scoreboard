//! Wall-clock time via SNTP, shown alongside the Match on the physical
//! display — grilled and recorded in
//! `docs/slices/02-display-bringup-plan.md`'s Checkpoint H.
//!
//! Time itself isn't domain state (see `display_render`'s module docs for
//! why it's kept out of that crate entirely) and doesn't come from
//! `MatchState` — it comes from the network, once, at a fixed
//! `Europe/London` offset. A full IANA timezone database is unnecessary
//! for a personal, single-timezone device; a fixed POSIX TZ rule (which
//! the OS's own `tzset()`/`localtime_r()` already understands, including
//! the BST/GMT DST transition dates) is all this needs.
//!
//! `time()`/`localtime_r()`/`tzset()` come from `esp_idf_svc::sys` (the
//! bindgen bindings against ESP-IDF's own newlib) rather than the `libc`
//! crate — `libc`'s xtensa-esp-idf target support doesn't expose
//! `tzset()`, even though the underlying C library actually has it.

use esp_idf_svc::sys::{localtime_r, time, time_t, tm, tzset};

const LONDON_TZ: &str = "GMT0BST,M3.5.0/1,M10.5.0";

pub struct Clock {
    sntp: esp_idf_svc::sntp::EspSntp<'static>,
}

impl Clock {
    /// Starts SNTP syncing in the background. Requires WiFi to already be
    /// up — SNTP has nothing to talk to otherwise (it'll just keep
    /// retrying quietly; `now_hh_mm()` reports `"--:--"` until it
    /// succeeds).
    pub fn start() -> anyhow::Result<Self> {
        // SAFETY: single-threaded at this point in startup (called once
        // from main() before any other thread exists) — setenv is not
        // thread-safe against concurrent getenv/setenv calls in general,
        // but there's no concurrency here to race with.
        unsafe {
            std::env::set_var("TZ", LONDON_TZ);
            tzset();
        }

        let sntp = esp_idf_svc::sntp::EspSntp::new_default()?;
        Ok(Self { sntp })
    }

    /// `"--:--"` until the first successful sync — see the module docs.
    pub fn now_hh_mm(&self) -> String {
        if self.sntp.get_sync_status() != esp_idf_svc::sntp::SyncStatus::Completed {
            return "--:--".to_string();
        }

        let mut now: time_t = 0;
        // SAFETY: `now` is a valid, in-scope local; `time()` only writes
        // through the pointer it's given.
        unsafe {
            time(&mut now);
        }

        let mut local = std::mem::MaybeUninit::<tm>::uninit();
        // SAFETY: `now` was just initialized above; `local` is
        // appropriately-sized uninitialized storage that localtime_r
        // fully initializes before returning (per POSIX) — the only
        // ESP-IDF-specific note is that tzset() (called in start()) must
        // have run first for LONDON_TZ to actually apply here, which it
        // has, since Clock can only exist via start().
        let parsed = unsafe {
            localtime_r(&now, local.as_mut_ptr());
            local.assume_init()
        };

        std::format!("{:02}:{:02}", parsed.tm_hour, parsed.tm_min)
    }
}
