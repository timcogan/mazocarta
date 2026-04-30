#[cfg(not(target_arch = "wasm32"))]
use std::env;
#[cfg(not(target_arch = "wasm32"))]
use std::process;

#[cfg(not(target_arch = "wasm32"))]
use mazocarta::sim::{SimulationConfig, run_simulations};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let config = parse_args(env::args().skip(1).collect());
    let stats = run_simulations(&config);
    print!("{}", stats.render_report());
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
fn parse_args(args: Vec<String>) -> SimulationConfig {
    let mut config = SimulationConfig::default();
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "--runs" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    usage_and_exit("Missing value for --runs.");
                };
                config.runs = parse_usize(value, "--runs");
            }
            "--seed-start" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    usage_and_exit("Missing value for --seed-start.");
                };
                config.seed_start = parse_u64(value, "--seed-start");
            }
            "--players" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    usage_and_exit("Missing value for --players.");
                };
                config.players = parse_usize(value, "--players").clamp(1, 2);
            }
            "--verbose" => {
                config.verbose = true;
            }
            "--help" | "-h" => {
                print_usage();
                process::exit(0);
            }
            other => usage_and_exit(&format!("Unknown flag: {other}")),
        }
        index += 1;
    }

    config
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_usize(value: &str, flag: &str) -> usize {
    value.parse::<usize>().unwrap_or_else(|_| {
        usage_and_exit(&format!("Invalid numeric value for {flag}: {value}"));
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_u64(value: &str, flag: &str) -> u64 {
    value.parse::<u64>().unwrap_or_else(|_| {
        usage_and_exit(&format!("Invalid numeric value for {flag}: {value}"));
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn usage_and_exit(message: &str) -> ! {
    eprintln!("{message}");
    print_usage();
    process::exit(2);
}

#[cfg(not(target_arch = "wasm32"))]
fn print_usage() {
    eprintln!(
        "Usage: cargo run --bin actor -- [--runs N] [--seed-start N] [--players 1|2] [--verbose]"
    );
}
