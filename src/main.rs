mod app;
mod clipboard;
mod config;
mod keys;
mod mandrill;
mod ui;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "mnml-msg-mandrill",
    version,
    about = "Mandrill (Mailchimp Transactional) browser for mnml"
)]
struct Cli {
    /// Print the resolved config + auth state and exit.
    #[arg(long)]
    check: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.check {
        let cfg = config::load();
        let auth = mandrill::Auth::from_env();

        println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        println!("config: {}", config::config_path().display());
        match &cfg {
            Ok(cfg) => {
                println!(
                    "refresh_interval_secs={} · messages_lookback_days={}",
                    cfg.refresh_interval_secs, cfg.messages_lookback_days
                );
                println!("tabs:");
                for (i, t) in cfg.tabs.iter().enumerate() {
                    println!("  {} ({}): kind={}", i + 1, t.name, t.kind);
                }
            }
            Err(e) => println!("config: ERROR — {e}"),
        }

        println!();
        println!("env: MANDRILL_API_KEY={}", mask_env("MANDRILL_API_KEY"));

        match &auth {
            Ok(a) => {
                println!();
                println!("api base: {}", a.api_base());
                print!("auth: ");
                // /users/ping.json is the canonical liveness check.
                match mandrill::ping(a) {
                    Ok(body) => {
                        let trimmed = body.trim_matches('"').trim();
                        println!("ok ({trimmed})");
                    }
                    Err(e) => {
                        println!("ERROR — {e}");
                        std::process::exit(2);
                    }
                }
            }
            Err(e) => {
                println!();
                println!("auth: ERROR — {e}");
                std::process::exit(2);
            }
        }
        if cfg.is_err() {
            std::process::exit(2);
        }
        return Ok(());
    }

    let cfg = config::load()?;
    let auth = match mandrill::Auth::from_env() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            eprintln!();
            eprintln!("setup:");
            eprintln!(
                "  export MANDRILL_API_KEY=...     (from Mandrill: Settings → SMTP & API Info → API Keys)"
            );
            eprintln!();
            eprintln!("then re-run, or `mnml-msg-mandrill --check` to confirm.");
            std::process::exit(2);
        }
    };

    let mut app = app::App::new(cfg, auth)?;
    ui::run(&mut app)
}

fn mask_env(name: &str) -> String {
    match std::env::var(name) {
        Ok(v) if !v.is_empty() => {
            if v.len() > 6 {
                format!("set ({} chars, ends …{})", v.len(), &v[v.len() - 4..])
            } else {
                format!("set ({} chars)", v.len())
            }
        }
        _ => "(unset)".into(),
    }
}
