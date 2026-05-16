//! GTK4 Settings dialog for LLM refiner configuration.
//!
//! Mirrors macOS `SettingsWindow.swift`: three Entry widgets (Base URL,
//! API Key, Model) + a status label + Test/Save buttons. Reads from
//! `AppState` on open; writes back via `AppState::update()` on Save.
//!
//! The Test button is wired in Task 5.8 — currently it just sets a
//! placeholder status string.

use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, Entry, Grid, Label, Orientation,
    PasswordEntry,
};

use crate::state::AppState;

pub fn build_window(app: &Application, state: &AppState) -> ApplicationWindow {
    let snap = state.snapshot();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("LLM Refinement Settings")
        .default_width(520)
        .default_height(260)
        .resizable(false)
        .build();

    let content = GtkBox::new(Orientation::Vertical, 16);
    content.set_margin_top(20);
    content.set_margin_bottom(20);
    content.set_margin_start(20);
    content.set_margin_end(20);

    let grid = Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(10);

    // Row 0: API Base URL
    let url_label = Label::new(Some("API Base URL:"));
    url_label.set_halign(Align::End);
    let url_entry = Entry::builder()
        .text(snap.llm_api_base_url.as_str())
        .placeholder_text("https://api.openai.com/v1")
        .hexpand(true)
        .build();
    grid.attach(&url_label, 0, 0, 1, 1);
    grid.attach(&url_entry, 1, 0, 1, 1);

    // Row 1: API Key (masked with peek icon)
    let key_label = Label::new(Some("API Key:"));
    key_label.set_halign(Align::End);
    let key_entry = PasswordEntry::builder()
        .text(snap.llm_api_key.as_str())
        .placeholder_text("sk-...")
        .show_peek_icon(true)
        .hexpand(true)
        .build();
    grid.attach(&key_label, 0, 1, 1, 1);
    grid.attach(&key_entry, 1, 1, 1, 1);

    // Row 2: Model
    let model_label = Label::new(Some("Model:"));
    model_label.set_halign(Align::End);
    let model_entry = Entry::builder()
        .text(snap.llm_model.as_str())
        .placeholder_text("gpt-4o-mini")
        .hexpand(true)
        .build();
    grid.attach(&model_label, 0, 2, 1, 1);
    grid.attach(&model_entry, 1, 2, 1, 1);

    content.append(&grid);

    // Status label (populated by Test in Task 5.8)
    let status_label = Label::new(None);
    status_label.set_halign(Align::Start);
    status_label.set_wrap(true);
    status_label.set_xalign(0.0);
    content.append(&status_label);

    // Button row — right-aligned
    let button_row = GtkBox::new(Orientation::Horizontal, 8);
    button_row.set_halign(Align::End);
    let test_button = Button::with_label("Test");
    let save_button = Button::with_label("Save");
    save_button.add_css_class("suggested-action");
    button_row.append(&test_button);
    button_row.append(&save_button);
    content.append(&button_row);

    window.set_child(Some(&content));

    // Save: persist all three fields via AppState::update, then close.
    {
        let state = state.clone();
        let window_for_save = window.clone();
        let url_entry = url_entry.clone();
        let key_entry = key_entry.clone();
        let model_entry = model_entry.clone();
        save_button.connect_clicked(move |_| {
            let url = url_entry.text().to_string();
            let key = key_entry.text().to_string();
            let model = model_entry.text().to_string();
            let result = state.update(|cfg| {
                cfg.llm_api_base_url = url.clone();
                cfg.llm_api_key = key.clone();
                cfg.llm_model = model.clone();
            });
            match result {
                Ok(()) => {
                    tracing::info!("settings: saved");
                    window_for_save.close();
                }
                Err(e) => {
                    tracing::error!(error = %e, "settings: save failed");
                }
            }
        });
    }

    // Test: placeholder for Task 5.8 — no HTTP yet.
    {
        let status_label = status_label.clone();
        test_button.connect_clicked(move |_| {
            status_label.set_text("Test not yet implemented (Task 5.8)");
        });
    }

    window
}
