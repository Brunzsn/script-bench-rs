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
            for case in script_bench::interop::INTEROP_CASES {
                println!("{}", case.name);
            }
        }
        Some("wasm") => {
            for case in script_bench::cases::CASES {
                if let Some((source, module)) = case.scripts.wasm() {
                    println!("{source}\t{module}");
                }
            }
        }
        Some(other) => {
            eprintln!("unknown argument: {other}");
            std::process::exit(2);
        }
    }
}
