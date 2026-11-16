use std::process::ExitCode;

fn main() -> ExitCode {
    snapapi::cli::run(std::env::args_os()).into()
}
