use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::screenshot::Screenshot;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    #[serde(skip)]
    last_error: Option<String>,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,

    #[serde(skip)]
    screenshoter: crate::screenshot::Watcher,

    timelapse_folder: PathBuf,

    high_res: bool,
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
            // Example stuff:
            label: "Hello World!".to_owned(),
            last_error: None,
            value: 2.7,
            screenshoter: crate::screenshot::Watcher::try_new().unwrap(),
            timelapse_folder,
            high_res: true,
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

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("eframe template");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });

            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }

            ui.checkbox(&mut self.high_res, "High Resolution");

            if ui.button("Screenshot").clicked() {
                // This is where you would call your own code to take a screenshot.
                // Here we just print a message to the console:
                let screenshot = self.screenshoter.take_screenshot(self.high_res).unwrap();
                log::info!("Took screenshot: {:?}", screenshot);
                dbg!(store_screenshot(screenshot, true, &self.timelapse_folder).unwrap());
            }

            ui.separator();

            ui.collapsing("Logs", |ui| {
                egui_logger::logger_ui(ui);
            });

            if let Some(error) = &self.last_error {
                ui.colored_label(egui::Color32::RED, error);
            }

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

fn store_screenshot(
    screenshot: Screenshot,
    remove_original: bool,
    folder: &Path,
) -> Result<PathBuf> {
    let now = chrono::Local::now();
    let folder = folder.join(format!(
        "{} {}",
        now.format("%Y-%m-%d"),
        screenshot.location,
    ));
    std::fs::create_dir_all(&folder)?;
    let filename = now.format("%H-%M-%S.bmp").to_string();
    let destination = folder.join(filename);
    std::fs::copy(&screenshot.path, &destination)?;

    if remove_original {
        std::fs::remove_file(screenshot.path)?;
    }

    Ok(destination)
}
