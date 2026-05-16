//! Shared application state across tray, listen loop, and Settings dialog.
//!
//! `AppState` owns the live `Config` behind a `parking_lot::Mutex` and
//! exposes `Notify` channels for state-change events:
//! - `shutdown`: Ctrl+C / tray Quit → both backend and GTK wind down.
//! - `config_changed`: Settings save / tray toggle → listen loop rebuilds
//!   its `LlmRefiner` on the next iteration.
//!
//! Why `parking_lot::Mutex` over `std::sync::Mutex`: no lock poisoning
//! (resilient to panics in holders), no `PoisonError` to thread through
//! `?`. API identical for our usage.

use std::sync::Arc;

use parking_lot::Mutex;
use tokio::sync::Notify;

use crate::config::Config;
use crate::error::{AppError, AppResult};

#[derive(Clone)]
pub struct AppState {
    config: Arc<Mutex<Config>>,
    pub shutdown: Arc<Notify>,
    pub config_changed: Arc<Notify>,
}

impl AppState {
    pub fn new(cfg: Config) -> Self {
        Self {
            config: Arc::new(Mutex::new(cfg)),
            shutdown: Arc::new(Notify::new()),
            config_changed: Arc::new(Notify::new()),
        }
    }

    /// Snapshot the current Config. Cheap clone (the struct is small).
    pub fn snapshot(&self) -> Config {
        self.config.lock().clone()
    }

    /// Mutate the config and persist to disk. Notifies `config_changed`
    /// on success so listeners can react (e.g., rebuild the refiner).
    pub fn update<F>(&self, mutator: F) -> AppResult<()>
    where
        F: FnOnce(&mut Config),
    {
        let snapshot = {
            let mut guard = self.config.lock();
            mutator(&mut guard);
            guard.clone()
        };
        snapshot
            .save()
            .map_err(|e| AppError::Config(format!("persist on update: {}", e)))?;
        self.config_changed.notify_waiters();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_reflects_current_state() {
        let cfg = Config {
            language_hint: "ja".into(),
            ..Config::default()
        };
        let state = AppState::new(cfg);
        assert_eq!(state.snapshot().language_hint, "ja");
    }

    #[test]
    fn direct_mutex_mutation_visible_in_snapshot() {
        let state = AppState::new(Config::default());
        {
            let mut guard = state.config.lock();
            guard.enabled = false;
        }
        assert!(!state.snapshot().enabled);
    }
}
