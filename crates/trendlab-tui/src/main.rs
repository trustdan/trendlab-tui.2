use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    match trendlab_tui::run_from_args(env::args().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}
