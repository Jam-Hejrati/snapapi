use std::process::ExitCode;

use snapapi::exit::Exit;

fn main() -> ExitCode {
    // Argument parsing arrives in Phase 7; until then only report the version.
    println!("snapapi {}", env!("CARGO_PKG_VERSION"));
    Exit::Ok.into()
}
