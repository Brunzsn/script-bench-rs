//! Prints the case registry for `bench.py`.
//!
//! - no argument: one case name per line
//! - `wasm`: `<source>\t<module>` per case with a wasm build

fn main() {
    match std::env::args().nth(1).as_deref() {
        None | Some("cases") => {
            for case in script_bench::cases::CASES {
                println!("{}", case.name);
            }
        }
        Some("wasm") => {
            for case in script_bench::cases::CASES {
                println!(
                    "{}\t{}",
                    case.scripts.wasm_source, case.scripts.wasm_module
                );
            }
        }
        Some(other) => {
            eprintln!("unknown argument: {other}");
            std::process::exit(2);
        }
    }
}
