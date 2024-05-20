use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use egui::{ProgressBar, Slider, SliderOrientation};

use crate::timelapse::{self, TimelapseControl};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    #[serde(skip)]
    screenshoter: crate::screenshot::Watcher,

    #[serde(skip)]
    current_timelapse: Option<TimelapseControl>,

    timelapse_folder: PathBuf,

    duration_seconds: u64,

    high_res: bool,

    remove_original: bool,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let timelapse_folder = directories::UserDirs::new()
            .unwrap()
            .picture_dir()
            .unwrap()
            .to_owned()
            .join("Elite Dangerous Timelapses");
        Self {
            screenshoter: crate::screenshot::Watcher::try_new().unwrap(),
            duration_seconds: 5,
            timelapse_folder,
            high_res: true,
            remove_original: true,
            current_timelapse: None,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // repaint always, to account for external threads update
        ctx.request_repaint();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Elite Dangerous Timelapse");

            if let Some(current_timelapse) = &mut self.current_timelapse {
                current_timelapse.update_status();
                match current_timelapse.status {
                    timelapse::Status::Capturing => {
                        ui.label("Capturing...");
                        ui.spinner();
                    }
                    timelapse::Status::Waiting(next) => {
                        ui.label(format!(
                            "Next in {}s",
                            1 + (next - Instant::now()).as_secs()
                        ));
                        ui.add(ProgressBar::new(
                            (next - Instant::now()).as_secs_f32() / self.duration_seconds as f32,
                        ));
                    }
                }
                if ui.button("Stop Timelapse").clicked() {
                    current_timelapse.stop();
                    self.current_timelapse = None;
                }
            } else {
                ui.add(
                    Slider::new(&mut self.duration_seconds, 1..=3600)
                        .logarithmic(true)
                        .clamp_to_range(true)
                        .smart_aim(true)
                        .orientation(SliderOrientation::Horizontal)
                        .trailing_fill(true)
                        .custom_formatter(|x, _| {
                            let x = x as u64;
                            if x < 60 {
                                format!("{}s", x)
                            } else {
                                format!("{}m{}s", x / 60, x % 60)
                            }
                        }),
                );
                ui.checkbox(&mut self.high_res, "High Resolution");
                ui.checkbox(&mut self.remove_original, "Remove Original");
                if ui.button("Start Timelapse").clicked() {
                    self.current_timelapse = match TimelapseControl::start(
                        self.timelapse_folder.clone(),
                        Duration::from_secs(self.duration_seconds),
                        self.high_res,
                        true,
                    ) {
                        Ok(timelapse) => Some(timelapse),
                        Err(e) => {
                            log::error!("Failed to start timelapse: {}", e);
                            None
                        }
                    };
                }

                if ui.button("Screenshot").clicked() {
                    if let Err(e) = timelapse::take_screenshot(
                        &mut self.screenshoter,
                        self.high_res,
                        self.remove_original,
                        &self.timelapse_folder,
                    ) {
                        log::error!("Failed to take screenshot: {}", e);
                    }
                }
            }

            ui.separator();

            ui.collapsing("Logs", |ui| {
                egui_logger::logger_ui(ui);
            });

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
