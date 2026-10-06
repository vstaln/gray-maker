//! gray-maker entry point: shell CLI with arguments, NDJSON sidecar without.

use std::io::{BufRead, Write};
use std::path::PathBuf;

use gray_maker::{dispatch, manifest};
use serde_json::{Value, json};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        if let Err(e) = sidecar() {
            eprintln!("gray-maker sidecar: {e}");
            std::process::exit(1);
        }
        return;
    }
    if args[0] == "manifest" {
        println!("{}", manifest::manifest());
        return;
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    match dispatch(&args, &cwd) {
        Ok(text) => println!("{text}"),
        Err(e) => {
            eprintln!("error: {e:#}");
            std::process::exit(if e.to_string().starts_with("unknown command") {
                2
            } else {
                1
            });
        }
    }
}

fn sidecar() -> std::io::Result<()> {
    let mut stdout = std::io::stdout();
    for line in std::io::stdin().lock().lines() {
        let Ok(req) = serde_json::from_str::<Value>(&line?) else {
            continue;
        };
        let method = req["method"].as_str().unwrap_or("");
        let Some(id) = req.get("id").cloned() else {
            if method == "plugin/shutdown" {
                break;
            }
            continue;
        };
        let (frame, exit) = match method {
            "plugin/manifest" => (json!({"id": id, "result": manifest::manifest()}), false),
            "command/run" => (
                json!({"id": id, "result": gray_maker::slash(&req["params"])}),
                false,
            ),
            "plugin/shutdown" => (json!({"id": id, "result": {}}), true),
            _ => (
                json!({"id": id, "error": {"code": -32601, "message": "method not found"}}),
                false,
            ),
        };
        writeln!(stdout, "{frame}")?;
        stdout.flush()?;
        if exit {
            break;
        }
    }
    Ok(())
}
