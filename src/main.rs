#![windows_subsystem = "windows"]

use config::Config;
use eframe::egui;
use injector::{is_elevated, Injector};
use std::path::PathBuf;
use std::sync::Arc;

const MAX_LOGS: usize = 1000;
const PROCESS_LIST_SCROLL_HEIGHT: f32 = 400.0;
const LOG_SCROLL_HEIGHT: f32 = 250.0;
const WINDOW_WIDTH: f32 = 700.0;
const WINDOW_HEIGHT: f32 = 700.0;
const MIN_WINDOW_WIDTH: f32 = 600.0;
const MIN_WINDOW_HEIGHT: f32 = 600.0;

const FONT_SIZE_LARGE: f32 = 18.0;
const FONT_SIZE_MEDIUM: f32 = 16.0;
const FONT_SIZE_NORMAL: f32 = 14.0;
const FONT_SIZE_SMALL: f32 = 12.0;

const ANIMATION_SPEED: f32 = 0.15;

mod config;
mod injector;

struct LogManager {
    messages: Vec<String>,
    new_log_alpha: f32,
    error_indices: Vec<usize>,
    max_logs: usize,
}

impl LogManager {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
            new_log_alpha: 0.0,
            error_indices: Vec::new(),
            max_logs: MAX_LOGS,
        }
    }

    fn add_log(&mut self, message: String) {
        self.new_log_alpha = 0.0;
        let is_error = message.to_lowercase().contains("error");
        self.messages.push(message);
        if is_error {
            self.error_indices.push(self.messages.len() - 1);
        }
        if self.messages.len() > self.max_logs {
            self.messages.remove(0);
            self.error_indices = self
                .error_indices
                .iter()
                .filter_map(|&i| i.checked_sub(1))
                .collect();
        }
    }

    fn get_messages(&self) -> &[String] {
        &self.messages
    }

    fn is_error(&self, index: usize) -> bool {
        self.error_indices.contains(&index)
    }

    fn update_frame(&mut self) {
        self.new_log_alpha = (self.new_log_alpha + ANIMATION_SPEED).min(1.0);
    }
}

struct InjectorApp {
    dll_path: Option<PathBuf>,
    process_name: String,
    auto_inject: bool,
    logger: LogManager,
    injector: Arc<Injector>,
    config: Config,
    all_processes: Vec<(String, u32)>,
    search_query: String,
    auto_injected: bool,
    show_process_list: bool,
    selected_pid: Option<u32>,
    injection_history: Vec<String>,
    fade_alpha: f32,
    panel_y_offset: f32,
    window_scale: f32,
}

impl Default for InjectorApp {
    fn default() -> Self {
        let config = Config::load();
        Self {
            dll_path: config.dll_path.as_ref().map(PathBuf::from),
            process_name: config.last_process.clone().unwrap_or_default(),
            auto_inject: config.auto_inject,
            logger: LogManager::new(),
            injector: Arc::new(Injector::new()),
            config,
            all_processes: Vec::new(),
            search_query: String::new(),
            auto_injected: false,
            show_process_list: false,
            selected_pid: None,
            injection_history: Vec::new(),
            fade_alpha: 0.0,
            panel_y_offset: 50.0,
            window_scale: 0.0,
        }
    }
}

impl InjectorApp {
    fn refresh_process_list(&mut self) {
        let processes = self.injector.get_all_processes();
        self.all_processes = processes.into_iter().map(|p| (p.name, p.pid)).collect();
    }

    fn add_log(&mut self, message: String) {
        self.logger.add_log(message);
    }

    fn is_process_running(&self) -> bool {
        if self.process_name.is_empty() {
            return false;
        }
        self.injector
            .get_all_processes()
            .iter()
            .any(|p| p.name.eq_ignore_ascii_case(&self.process_name))
    }

    fn inject_dll(&mut self) {
        let dll_path = match &self.dll_path {
            Some(path) => path.clone(),
            None => {
                self.add_log("Error: No DLL selected".to_string());
                return;
            }
        };

        let proc_name = self.process_name.clone();
        if proc_name.is_empty() {
            self.add_log("Error: No process name specified".to_string());
            return;
        }

        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        self.add_log(format!("Attempting to inject into {}...", proc_name));

        match self.injector.inject(&proc_name, &dll_path) {
            Ok(_) => {
                self.add_log("Injection successful!".to_string());
                self.auto_injected = true;
                self.config.last_process = Some(proc_name.clone());
                self.config.save();
                let dll_name = dll_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown.dll");
                self.injection_history
                    .insert(0, format!("[{}] {} -> {}", timestamp, proc_name, dll_name));
                if self.injection_history.len() > 10 {
                    self.injection_history.pop();
                }
            }
            Err(e) => {
                self.add_log(format!("Injection failed: {}", e));
                self.auto_injected = false;
            }
        }
    }

    fn check_auto_inject(&mut self) {
        if self.auto_inject
            && !self.process_name.is_empty()
            && !self.auto_injected
            && self.is_process_running()
            && self.dll_path.is_some()
        {
            self.add_log("Auto-inject: Target process detected, injecting...".to_string());
            self.inject_dll();
        }
    }
}

impl eframe::App for InjectorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.fade_alpha = self.fade_alpha + (1.0 - self.fade_alpha) * ANIMATION_SPEED;
        self.panel_y_offset = self.panel_y_offset + (0.0 - self.panel_y_offset) * ANIMATION_SPEED;

        let target_scale = if self.show_process_list { 1.0 } else { 0.0 };
        self.window_scale =
            self.window_scale + (target_scale - self.window_scale) * ANIMATION_SPEED;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(self.panel_y_offset + 20.0);

            ui.vertical_centered(|ui| {
                let title_color = egui::Color32::from_rgba_premultiplied(
                    255,
                    255,
                    255,
                    (255.0 * self.fade_alpha) as u8,
                );
                ui.label(
                    egui::RichText::new("Ruin DLL Injector")
                        .size(FONT_SIZE_LARGE)
                        .color(title_color),
                );
                ui.label(
                    egui::RichText::new(if is_elevated() {
                        "Administrator"
                    } else {
                        "Not Administrator"
                    })
                    .size(FONT_SIZE_SMALL)
                    .color(if is_elevated() {
                        egui::Color32::LIGHT_GREEN
                    } else {
                        egui::Color32::LIGHT_RED
                    }),
                );
            });

            ui.add_space(20.0);

            ui.separator();
            ui.add_space(20.0);

            ui.label(
                egui::RichText::new("Target DLL")
                    .size(FONT_SIZE_MEDIUM)
                    .color(egui::Color32::WHITE),
            );
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                let display_text = match &self.dll_path {
                    Some(path) => path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Select a DLL...")
                        .to_string(),
                    None => "No DLL selected".to_string(),
                };
                ui.add_enabled_ui(false, |ui| {
                    ui.text_edit_singleline(&mut display_text.to_owned());
                });

                if ui.button("Browse").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("DLL", &["dll"])
                        .pick_file()
                    {
                        self.dll_path = Some(path.clone());
                        self.config.dll_path = Some(path.display().to_string());
                        self.config.save();
                        self.add_log(format!("Selected DLL: {}", path.display()));
                    }
                }
            });

            if let Some(path) = &self.dll_path {
                ui.label(
                    egui::RichText::new(path.display().to_string())
                        .size(FONT_SIZE_SMALL)
                        .color(egui::Color32::LIGHT_GRAY),
                );
            }

            ui.add_space(20.0);

            ui.label(
                egui::RichText::new("Target Process")
                    .size(FONT_SIZE_MEDIUM)
                    .color(egui::Color32::WHITE),
            );
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.process_name);

                let btn_text = if self.is_process_running() {
                    "Running"
                } else {
                    "List"
                };
                if ui.button(btn_text).clicked() {
                    self.refresh_process_list();
                    self.show_process_list = true;
                }
            });

            if !self.process_name.is_empty() {
                let status = if self.is_process_running() {
                    "Status: Running".to_string()
                } else {
                    "Status: Not found".to_string()
                };
                ui.label(egui::RichText::new(status).size(FONT_SIZE_SMALL).color(
                    if self.is_process_running() {
                        egui::Color32::LIGHT_GREEN
                    } else {
                        egui::Color32::LIGHT_GRAY
                    },
                ));
            }

            ui.add_space(20.0);

            ui.separator();
            ui.add_space(20.0);

            ui.horizontal(|ui| {
                let inject_button = egui::Button::new("Inject DLL");
                if ui.add(inject_button).clicked() {
                    self.inject_dll();
                }

                let prev_auto = self.auto_inject;
                if ui.checkbox(&mut self.auto_inject, "Auto-inject").changed() {
                    self.config.auto_inject = self.auto_inject;
                    self.config.save();
                    if self.auto_inject {
                        self.auto_injected = false;
                        self.add_log(
                            "Auto-inject enabled. Will inject when target process is detected."
                                .to_string(),
                        );
                    } else {
                        self.add_log("Auto-inject disabled.".to_string());
                    }
                }

                if self.auto_inject != prev_auto && self.auto_inject {
                    ui.label(egui::RichText::new("Active").color(egui::Color32::LIGHT_GREEN));
                }
            });

            ui.add_space(20.0);

            if !self.injection_history.is_empty() {
                ui.label(
                    egui::RichText::new("Recent Injections")
                        .size(FONT_SIZE_MEDIUM)
                        .color(egui::Color32::WHITE),
                );
                ui.add_space(10.0);

                for entry in &self.injection_history {
                    ui.label(
                        egui::RichText::new(entry)
                            .size(FONT_SIZE_NORMAL)
                            .color(egui::Color32::LIGHT_GRAY),
                    );
                }
                ui.add_space(20.0);
            }

            ui.separator();
            ui.add_space(20.0);

            ui.label(
                egui::RichText::new("Activity Log")
                    .size(FONT_SIZE_MEDIUM)
                    .color(egui::Color32::WHITE),
            );
            ui.add_space(10.0);

            egui::ScrollArea::vertical()
                .max_height(LOG_SCROLL_HEIGHT)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for (i, log) in self.logger.get_messages().iter().enumerate() {
                        let is_error = self.logger.is_error(i);
                        let is_new_log = i == self.logger.get_messages().len() - 1;

                        let base_color = if is_error {
                            egui::Color32::LIGHT_RED
                        } else if is_new_log {
                            egui::Color32::LIGHT_GREEN
                        } else {
                            egui::Color32::LIGHT_GRAY
                        };

                        let alpha = if is_new_log {
                            self.logger.new_log_alpha
                        } else {
                            1.0
                        };
                        let color = egui::Color32::from_rgba_premultiplied(
                            base_color.r(),
                            base_color.g(),
                            base_color.b(),
                            (255.0 * alpha) as u8,
                        );

                        ui.label(egui::RichText::new(log).size(FONT_SIZE_NORMAL).color(color));
                    }
                    self.logger.update_frame();
                });

            ui.add_space(15.0);

            ui.label(
                egui::RichText::new("Some processes require administrator privileges")
                    .size(FONT_SIZE_SMALL)
                    .color(egui::Color32::LIGHT_GRAY),
            );

            self.check_auto_inject();
        });

        if self.window_scale > 0.01 {
            let window_alpha = self.window_scale;
            egui::Window::new("Select Process")
                .collapsible(false)
                .resizable(true)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.set_enabled(self.window_scale > 0.5);

                    let title_color = egui::Color32::from_rgba_premultiplied(
                        255,
                        255,
                        255,
                        (255.0 * window_alpha) as u8,
                    );
                    ui.heading(egui::RichText::new("Running Processes").color(title_color));
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Search: ");
                        ui.text_edit_singleline(&mut self.search_query);
                    });

                    ui.separator();

                    let search_lower = self.search_query.to_lowercase();
                    let mut matched_process: Option<(String, u32)> = None;

                    egui::ScrollArea::vertical()
                        .max_height(PROCESS_LIST_SCROLL_HEIGHT)
                        .show(ui, |ui| {
                            let mut has_matches = false;
                            for (name, pid) in &self.all_processes {
                                let name_lower = name.to_lowercase();
                                if search_lower.is_empty() || name_lower.contains(&search_lower) {
                                    has_matches = true;

                                    if ui.button(format!("{} (PID: {})", name, pid)).clicked() {
                                        matched_process = Some((name.clone(), *pid));
                                    }
                                }
                            }

                            if !has_matches {
                                ui.label("No matching processes found");
                            }
                        });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show_process_list = false;
                            self.search_query.clear();
                        }
                    });

                    if let Some((name, pid)) = matched_process {
                        self.process_name = name.clone();
                        self.selected_pid = Some(pid);
                        self.add_log(format!("Selected process: {}", name));
                        self.show_process_list = false;
                        self.search_query.clear();
                    }
                });
        }

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_min_inner_size([MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT])
            .with_title("Ruin DLL Injector"),
        ..Default::default()
    };

    eframe::run_native(
        "Ruin DLL Injector",
        options,
        Box::new(|_cc| Box::<InjectorApp>::default()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_manager_add_message() {
        let mut logger = LogManager::new();
        logger.add_log("Test message".to_string());

        assert_eq!(logger.get_messages().len(), 1);
        assert_eq!(logger.get_messages()[0], "Test message");
    }

    #[test]
    fn test_log_manager_error_detection() {
        let mut logger = LogManager::new();
        logger.add_log("Error: something went wrong".to_string());
        logger.add_log("Normal message".to_string());

        assert!(logger.is_error(0));
        assert!(!logger.is_error(1));
    }

    #[test]
    fn test_log_manager_new_status() {
        let mut logger = LogManager::new();
        logger.add_log("Test".to_string());

        assert_eq!(logger.new_log_alpha, 0.0);

        for _ in 0..120 {
            logger.update_frame();
        }

        assert!(logger.new_log_alpha >= 1.0);
    }

    #[test]
    fn test_log_manager_max_limit() {
        let mut logger = LogManager::new();

        for i in 0..1002 {
            logger.add_log(format!("Log {}", i));
        }

        assert_eq!(logger.get_messages().len(), MAX_LOGS);
        assert!(!logger.get_messages().contains(&"Log 0".to_string()));
        assert!(!logger.get_messages().contains(&"Log 1".to_string()));
        assert!(logger.get_messages().contains(&"Log 2".to_string()));
        assert!(logger.get_messages().contains(&"Log 1001".to_string()));
    }
}
