use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let response = trendlab_cli::dispatch(env::args().skip(1));

    if !response.stdout.is_empty() {
        println!("{}", response.stdout);
    }

    if !response.stderr.is_empty() {
        eprintln!("{}", response.stderr);
    }

    ExitCode::from(response.exit_code)
}
