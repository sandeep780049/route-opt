//! Batch runner CLI.
//!
//! ```text
//! route_engine scenario.json              # solve one scenario, print answer
//! route_engine --report examples/         # aggregate Markdown stats over a corpus
//! ```

mod engine;
mod json;

use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: route_engine <scenario.json | --report <dir>>");
        std::process::exit(2);
    }
    if args[1] == "--report" {
        let dir = PathBuf::from(
            args.get(2)
                .map(String::as_str)
                .unwrap_or("examples"),
        );
        match engine::load_scenarios(&dir) {
            Ok(list) => print!("{}", engine::report(&list)),
            Err(err) => {
                eprintln!("route_engine: {err}");
                std::process::exit(1);
            }
        }
        return;
    }
    let path = PathBuf::from(&args[1]);
    let scenarios = match engine::load_scenarios(&path) {
        Ok(list) => list,
        Err(_) => match engine::load_scenarios(&path.parent().unwrap_or(&path)) {
            Ok(all) => all
                .into_iter()
                .filter(|s| {
                    path.file_stem()
                        .map(|f| f.to_string_lossy() == s.name)
                        .unwrap_or(false)
                })
                .collect(),
            Err(err) => {
                eprintln!("route_engine: {err}");
                std::process::exit(1);
            }
        },
    };
    for sc in &scenarios {
        println!("{}", engine::answer_to_string(&engine::solve(sc)));
    }
}
