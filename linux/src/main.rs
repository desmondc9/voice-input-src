use std::sync::Arc;

use anyhow::Context;
use ksni::TrayMethods;
use tokio::sync::Notify;
use voice_input::{config::Config, tray::VoiceInputTray};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cfg = Config::load().context("loading config")?;
    tracing::info!(
        language_hint = %cfg.language_hint,
        llm_enabled = cfg.llm_enabled,
        whisper_model_size = %cfg.whisper_model_size,
        "config loaded"
    );

    // Materialize the config file on disk so the user can edit it.
    cfg.save().context("persisting config defaults")?;

    let shutdown = Arc::new(Notify::new());

    let tray = VoiceInputTray::new(shutdown.clone());
    let _tray_handle = tray.spawn().await.context("spawning tray")?;

    tracing::info!("voice-input running — Quit via tray icon or Ctrl+C");

    tokio::select! {
        _ = shutdown.notified() => tracing::info!("tray Quit received"),
        _ = tokio::signal::ctrl_c() => tracing::info!("SIGINT received"),
    }

    tracing::info!("shutdown complete");
    Ok(())
}
