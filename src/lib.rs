//! gray-maker: scaffold, check, build, release and publish gray plugins.
//!
//! Every step is a plain function returning a human-readable report, so the
//! shell CLI (`gray-maker <cmd>`) and the REPL command (`/maker <cmd>`) share
//! one implementation.

pub mod build;
pub mod check;
pub mod manifest;
pub mod project;
pub mod publish;
pub mod release;
pub mod run;
pub mod template;

use std::path::Path;

pub const USAGE: &str = "gray-maker — make gray plugins

usage: gray-maker <command> [args]

commands:
  new <name> [--dir D] [--description TEXT] [--no-repo]
                    scaffold ~/grayplugins/gray-<name> and its GitHub repo
  check             build-free sanity checks: one entry point, manifest handshake
  build [--remote HOST]
                    static x86_64 musl release build + reproducible tarball
  release           tag v<version>, upload the tarball, re-download and verify
  publish           submit the release to the gray registry (needs `gray account login`)
  ship [--remote HOST]
                    check → build → release → publish, stopping at the first failure
  manifest          print this plugin's manifest
  help              show this text

build/check/release/publish/ship act on the plugin in the current directory.
with no arguments, gray-maker runs the NDJSON sidecar protocol on stdio.";

/// Parsed command line shared by the shell CLI and `/maker`.
pub fn dispatch(args: &[String], cwd: &Path) -> anyhow::Result<String> {
    let cmd = args.first().map(String::as_str).unwrap_or("help");
    let rest = &args[args.len().min(1)..];
    let flag = |name: &str| -> Option<String> {
        rest.iter()
            .position(|a| a == name)
            .and_then(|i| rest.get(i + 1).cloned())
    };
    let has = |name: &str| rest.iter().any(|a| a == name);
    match cmd {
        "new" => {
            let name = rest
                .first()
                .filter(|a| !a.starts_with("--"))
                .ok_or_else(|| anyhow::anyhow!("usage: gray-maker new <name>"))?;
            template::new_plugin(&template::NewOpts {
                name: name.clone(),
                dir: flag("--dir").map(Into::into),
                description: flag("--description"),
                create_repo: !has("--no-repo"),
            })
        }
        "check" => check::check(&project::Project::load(cwd)?),
        "build" => build::build(&project::Project::load(cwd)?, flag("--remote").as_deref())
            .map(|b| b.report()),
        "release" => release::release(&project::Project::load(cwd)?),
        "publish" => publish::publish(&project::Project::load(cwd)?),
        "ship" => ship(cwd, flag("--remote").as_deref()),
        "help" | "-h" | "--help" => Ok(USAGE.to_string()),
        other => anyhow::bail!("unknown command '{other}'\n\n{USAGE}"),
    }
}

fn ship(cwd: &Path, remote: Option<&str>) -> anyhow::Result<String> {
    let p = project::Project::load(cwd)?;
    let mut out = vec![check::check(&p)?];
    out.push(build::build(&p, remote)?.report());
    out.push(release::release(&p)?);
    out.push(publish::publish(&p)?);
    Ok(out.join("\n"))
}

/// `/maker …` from the REPL. gray gives `command/run` 30s, so only `new`
/// runs inline; long steps hand the agent a prompt to run them in bash, where
/// it can read failures and fix the plugin.
pub fn slash(params: &serde_json::Value) -> serde_json::Value {
    let argv: Vec<String> = params["argv"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let cwd = params["session"]["cwd"]
        .as_str()
        .filter(|c| !c.is_empty())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    match argv.first().map(String::as_str) {
        None | Some("help") => {
            serde_json::json!({ "text": USAGE.replace("gray-maker ", "/maker ") })
        }
        Some("new") => {
            let text = match dispatch(&argv, &cwd) {
                Ok(t) => t,
                Err(e) => format!("maker new failed: {e:#}"),
            };
            serde_json::json!({ "text": text })
        }
        Some("check" | "build" | "release" | "publish" | "ship") => serde_json::json!({
            "prompt": format!(
                "Run `gray-maker {}` with bash in {} (it can take minutes). \
                 If it fails, read the error, fix the plugin, commit, and re-run. \
                 Report the result in one short paragraph.",
                argv.join(" "),
                cwd.display()
            ),
        }),
        Some(other) => {
            serde_json::json!({ "text": format!("unknown /maker command '{other}'\n\n{}", USAGE.replace("gray-maker ", "/maker ")) })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn slash_help_and_unknown_are_text() {
        assert!(
            slash(&json!({"argv": []}))["text"]
                .as_str()
                .unwrap()
                .contains("usage: /maker <command>")
        );
        assert!(
            slash(&json!({"argv": ["zap"]}))["text"]
                .as_str()
                .unwrap()
                .contains("unknown")
        );
    }

    #[test]
    fn slash_long_steps_become_agent_prompts_in_the_session_cwd() {
        let r =
            slash(&json!({"argv": ["ship", "--remote", "maid"], "session": {"cwd": "/p/gray-w"}}));
        let p = r["prompt"].as_str().unwrap();
        assert!(p.contains("`gray-maker ship --remote maid`") && p.contains("/p/gray-w"));
    }

    #[test]
    fn manifest_declares_maker() {
        let m = manifest::manifest();
        assert_eq!(m["name"], "maker");
        assert_eq!(m["commands"], json!(["/maker"]));
        assert_eq!(m["version"], env!("CARGO_PKG_VERSION"));
    }
}
