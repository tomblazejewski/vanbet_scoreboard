use std::path::Path;

fn main() {
    embuild::espidf::sysenv::output();

    // src/secrets.rs is gitignored (real WiFi credentials, never
    // committed), so it doesn't exist on a fresh clone or in CI. Stage
    // whichever of it / the committed secrets.rs.example exists into
    // OUT_DIR for main.rs to include!() — real credentials if present,
    // the harmless placeholder otherwise. Never touches src/secrets.rs
    // itself, so a local real file is never at risk of being overwritten.
    //
    // Two earlier approaches were tried and both broke `cargo fmt`:
    // #[cfg(feature = ...)] + #[path] switching (rustfmt doesn't
    // evaluate #[cfg(feature)] the way rustc does when picking which
    // #[path]-gated module to format), and seeding src/secrets.rs
    // directly from this same build.rs (cargo fmt never runs build
    // scripts, so the file still didn't exist when rustfmt looked for
    // it). include!() sidesteps both: unlike `mod name;`, rustfmt never
    // resolves or requires the existence of an include!()'d path — it's
    // just an ordinary macro call to it.
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let real = Path::new("src/secrets.rs");
    let source = if real.exists() {
        real
    } else {
        Path::new("src/secrets.rs.example")
    };
    std::fs::copy(source, Path::new(&out_dir).join("secrets.rs"))
        .expect("failed to stage secrets.rs for include!()");
    println!("cargo:rerun-if-changed=src/secrets.rs");
    println!("cargo:rerun-if-changed=src/secrets.rs.example");
}
