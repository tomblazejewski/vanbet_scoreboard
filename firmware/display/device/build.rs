use std::path::Path;

fn main() {
    embuild::espidf::sysenv::output();

    // src/secrets.rs is gitignored (real WiFi credentials, never
    // committed), so it doesn't exist on a fresh clone or in CI. A plain
    // `mod secrets;` in main.rs needs *some* file there to compile.
    //
    // A #[cfg(feature = ...)] + #[path] switch between secrets.rs and
    // secrets.rs.example was tried first, but rustfmt doesn't evaluate
    // `#[cfg(feature = ...)]` the way rustc does when picking which
    // #[path]-gated module to format, so it broke `cargo fmt` outright
    // (tried to read whichever variant it guessed, regardless of the
    // active feature set). Copying the placeholder into place here avoids
    // any cfg/path cleverness: real local credentials, once written to
    // secrets.rs, are never touched or overwritten by this.
    let secrets = Path::new("src/secrets.rs");
    if !secrets.exists() {
        std::fs::copy("src/secrets.rs.example", secrets)
            .expect("failed to seed src/secrets.rs from src/secrets.rs.example");
    }
    println!("cargo:rerun-if-changed=src/secrets.rs.example");
}
