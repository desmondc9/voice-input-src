use std::sync::Arc;

use ksni::{menu::StandardItem, MenuItem, Tray};
use tokio::sync::Notify;

/// KSNI tray with a single "Quit" item. Quit triggers `shutdown.notify_one()`
/// so `main` can perform an orderly shutdown.
pub struct VoiceInputTray {
    shutdown: Arc<Notify>,
}

impl VoiceInputTray {
    pub fn new(shutdown: Arc<Notify>) -> Self {
        Self { shutdown }
    }
}

impl Tray for VoiceInputTray {
    fn id(&self) -> String {
        "com.yetone.VoiceInput".into()
    }

    fn title(&self) -> String {
        "VoiceInput".into()
    }

    fn icon_name(&self) -> String {
        "audio-input-microphone".into()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: "VoiceInput".into(),
            description: "Hold the configured key to dictate".into(),
            icon_name: "audio-input-microphone".into(),
            icon_pixmap: Vec::new(),
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![StandardItem {
            label: "Quit".into(),
            icon_name: "application-exit".into(),
            activate: Box::new(|this: &mut Self| {
                tracing::info!("tray: Quit selected");
                this.shutdown.notify_one();
            }),
            ..Default::default()
        }
        .into()]
    }
}
