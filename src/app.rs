/// We derive Deserialize/Serialize so we can persist app state on shutdown.
use dittolive_ditto::{identity::*, prelude::*};
use std::{self, str::FromStr, sync::Arc};
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
// if we add new fields, give them default values when deserializing old state

pub struct TemplateApp {
    // Example stuff:
    label: String,
    app_id: String,
    app_token: String,

    // this how you opt-out of serialization of a member
    #[serde(skip)]
    value: f32,
    #[serde(skip)]
    show_settings_dialog: bool,
    #[serde(skip)]
    new_message: String,
    #[serde(skip)]
    ditto: Option<Ditto>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Dotti".to_owned(),
            value: 2.7,
            show_settings_dialog: false,
            app_id: "".to_string(),
            app_token: "".to_string(),
            new_message: "".to_string(),
            ditto: None,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        let root = Arc::new(PersistentRoot::from_current_exe().unwrap());
        let ditto = Ditto::builder()
            .with_root(root)
            .with_minimum_log_level(LogLevel::Debug)
            .with_identity(|ditto_root| {
                let app_id = AppId::from_str("").unwrap();
                let shared_token = "".to_string();
                let enable_cloud_sync = true;
                let custom_auth_url = None;
                OnlinePlayground::new(
                    ditto_root,
                    app_id,
                    shared_token,
                    enable_cloud_sync,
                    custom_auth_url,
                )
            })
            .unwrap()
            .build()
            .unwrap();

        ditto.start_sync();
        catppuccin_egui::set_theme(&cc.egui_ctx, catppuccin_egui::MOCHA);
        configure_text_styles(&cc.egui_ctx);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Default::default()
    }
}

use egui::{FontFamily, FontId, RichText, TextStyle};

#[inline]
fn heading2() -> TextStyle {
    TextStyle::Name("Heading2".into())
}

#[inline]
fn heading3() -> TextStyle {
    TextStyle::Name("ContextHeading".into())
}

fn configure_text_styles(ctx: &egui::Context) {
    use FontFamily::{Monospace, Proportional};

    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(25.0, Proportional)),
        (heading2(), FontId::new(22.0, Proportional)),
        (heading3(), FontId::new(19.0, Proportional)),
        (TextStyle::Body, FontId::new(16.0, Proportional)),
        (TextStyle::Monospace, FontId::new(12.0, Monospace)),
        (TextStyle::Button, FontId::new(12.0, Proportional)),
        (TextStyle::Small, FontId::new(8.0, Proportional)),
    ]
    .into();
    ctx.set_style(style);
}

use std::collections::HashMap;

fn messages() -> HashMap<String, String> {
    let mut messages = HashMap::new();
    messages.insert("joe".to_string(), "blah blah blah".to_string());
    messages.insert("billy".to_string(), "yeah, you too".to_string());
    messages.insert("joe".to_string(), "well, i dunno".to_string());

    messages
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                    if ui.button("Settings").clicked() {
                        self.show_settings_dialog = true;
                        ui.close_menu();
                    }
                });
            });
            if self.show_settings_dialog {
                egui::Window::new("Settings")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.vertical(|ui| {
                            let app_id_label = ui.label("APP_ID: ");

                            ui.text_edit_singleline(&mut self.app_id)
                                .labelled_by(app_id_label.id);

                            let app_token_label = ui.label("APP_TOKEN: ");
                            ui.text_edit_singleline(&mut self.app_token)
                                .labelled_by(app_token_label.id);

                            if ui.button("Save").clicked() {
                                self.show_settings_dialog = false;
                            }
                            // Dialog with APP ID and TOKEN
                        })
                    });
            }
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Rooms");

            ui.label(
                RichText::new("Public Rooms")
                    .text_style(heading2())
                    .strong(),
            );
            ui.add_space(5.);
            ui.label(RichText::new("PublicRoom").text_style(heading3()).strong());

            ui.add_space(15.);

            ui.label(
                RichText::new("Private Rooms")
                    .text_style(heading2())
                    .strong(),
            );

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("Ditto", "https://github.com/getditto/ditto");
                    ui.label(".");
                    ui.spacing_mut().item_spacing.x = 0.0;
                });
                ui.horizontal(|ui| {
                    egui::warn_if_debug_build(ui);
                });
            });
        });

        // Main Window area
        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            // Message bubbles area
            let msg_bubbles_layout =
                egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true);
            ui.allocate_ui_with_layout(ui.available_size(), msg_bubbles_layout, |ui| {
                let msgs = messages();
                for (name, message) in &msgs {
                    let label_string = format!("[{}] {}", name, message);
                    egui::Frame::none()
                        .rounding(5.0)
                        .inner_margin(5.0)
                        .fill(egui::Color32::BLUE)
                        .show(ui, |ui| ui.label(label_string));
                }
            });

            // Message entry area
            let msg_entry_layout =
                egui::Layout::bottom_up(egui::Align::LEFT).with_cross_justify(true);
            ui.allocate_ui_with_layout(ui.available_size(), msg_entry_layout, |ui| {
                ui.text_edit_singleline(&mut self.new_message);
            });
        });
    }
}
