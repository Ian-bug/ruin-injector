#![windows_subsystem = "windows"]

use config::Config;
use eframe::egui;
use injector::{Injector, ProcessInfo};
use std::path::PathBuf;
use std::sync::Arc;

const NEW_LOG_DURATION_FRAMES: usize = 120;
const MAX_LOGS: usize = 1000;
const PROCESS_LIST_SCROLL_HEIGHT: f32 = 300.0;
const LOG_SCROLL_HEIGHT: f32 = 150.0;
const WINDOW_WIDTH: f32 = 500.0;
const WINDOW_HEIGHT: f32 = 480.0;
const MIN_WINDOW_WIDTH: f32 = 350.0;
const MIN_WINDOW_HEIGHT: f32 = 400.0;

mod config;
mod injector;

struct LogManager {
    messages: Vec<String>,
    new_log_start_index: usize,
    new_log_frame_counter: usize,
    error_indices: Vec<usize>,
    max_logs: usize,
}

struct ProcessSelector {
    show_list: bool,
    all_processes: Vec<(String, u32)>,
    search_query: String,
}

impl ProcessSelector {
    fn new() -> Self {
        Self {
            show_list: false,
            all_processes: Vec::new(),
            search_query: String::new(),
        }
    }

    fn refresh(&mut self, processes: Vec<ProcessInfo>) {
        self.all_processes = processes.into_iter().map(|p| (p.name, p.pid)).collect();
    }
}

impl LogManager {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
            new_log_start_index: 0,
            new_log_frame_counter: 0,
            error_indices: Vec::new(),
            max_logs: MAX_LOGS,
        }
    }

    fn add_log(&mut self, message: String) {
        self.new_log_start_index = self.messages.len();
        self.new_log_frame_counter = 0;

        let is_error = message.to_lowercase().contains("error");
        self.messages.push(message);

        if is_error {
            self.error_indices.push(self.messages.len() - 1);
        }

        if self.messages.len() > self.max_logs {
            self.messages.remove(0);
            self.new_log_start_index = self.new_log_start_index.saturating_sub(1);
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

    fn is_new(&self, index: usize) -> bool {
        index >= self.new_log_start_index && self.new_log_frame_counter < NEW_LOG_DURATION_FRAMES
    }

    fn update_frame(&mut self) {
        if self.new_log_frame_counter < NEW_LOG_DURATION_FRAMES {
            self.new_log_frame_counter += 1;
        }
    }
}

struct InjectorApp {
    dll_path: Option<PathBuf>,
    process_name: String,
    auto_inject: bool,
    logger: LogManager,
    injector: Arc<Injector>,
    config: Config,
    selector: ProcessSelector,
    auto_injected: bool,
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
            selector: ProcessSelector::new(),
            auto_injected: false,
        }
    }
}

impl InjectorApp {
    fn refresh_process_list(&mut self) {
        let processes = self.injector.get_all_processes();
        self.selector.refresh(processes);
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

        self.add_log(format!("Attempting to inject into {}...", proc_name));

        match self.injector.inject(&proc_name, &dll_path) {
            Ok(_) => {
                self.add_log("Injection successful!".to_string());
                self.auto_injected = true;
                self.config.last_process = Some(proc_name);
                self.config.save();
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
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Ruin DLL Injector");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("DLL Path:");
                if ui.button("Browse...").clicked() {
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
                ui.label(path.display().to_string());
            } else {
                ui.label("No DLL selected");
            }

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Process Name:");
                ui.text_edit_singleline(&mut self.process_name);

                if ui.button("📋 Select Process").clicked() {
                    self.refresh_process_list();
                    self.selector.show_list = true;
                }
            });

            if self.selector.show_list {
                egui::Window::new("Select Process")
                    .collapsible(false)
                    .resizable(true)
                    .show(ctx, |ui| {
                        ui.heading("Running Processes");
                        ui.separator();

                        ui.horizontal(|ui| {
                            ui.label("🔍 Search:");
                            ui.text_edit_singleline(&mut self.selector.search_query);
                        });

                        ui.separator();

                        let search_lower = self.selector.search_query.to_lowercase();
                        let mut matched_process: Option<(String, u32)> = None;

                        egui::ScrollArea::vertical()
                            .max_height(PROCESS_LIST_SCROLL_HEIGHT)
                            .show(ui, |ui| {
                                let mut has_matches = false;
                                for (name, pid) in &self.selector.all_processes {
                                    let name_lower = name.to_lowercase();
                                    if search_lower.is_empty() || name_lower.contains(&search_lower)
                                    {
                                        has_matches = true;
                                        if ui
                                            .button(format!("Select: {} (PID: {})", name, pid))
                                            .clicked()
                                        {
                                            matched_process = Some((name.clone(), *pid));
                                        }
                                    }
                                }

                                if !has_matches {
                                    ui.label("No matching processes found");
                                }
                            });

                        ui.separator();
                        if ui.button("Cancel").clicked() {
                            self.selector.show_list = false;
                            self.selector.search_query.clear();
                        }

                        if let Some((name, _)) = matched_process {
                            self.process_name = name.clone();
                            self.add_log(format!("Selected process: {}", name));
                            self.selector.show_list = false;
                            self.selector.search_query.clear();
                        }
                    });
            }

            ui.horizontal(|ui| {
                if ui.button("Inject").clicked() {
                    self.inject_dll();
                }

                let prev_auto_inject = self.auto_inject;
                ui.checkbox(&mut self.auto_inject, "Auto Inject");
                if prev_auto_inject != self.auto_inject {
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
            });

            ui.separator();

            ui.heading("Logs");
            egui::ScrollArea::vertical()
                .max_height(LOG_SCROLL_HEIGHT)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for (i, log) in self.logger.get_messages().iter().enumerate() {
                        let is_new = self.logger.is_new(i);
                        let is_error = self.logger.is_error(i);

                        let color = if is_error {
                            egui::Color32::RED
                        } else if is_new {
                            egui::Color32::from_rgb(100, 255, 100)
                        } else {
                            egui::Color32::WHITE
                        };

                        ui.colored_label(color, log);
                    }

                    self.logger.update_frame();
                });

            ui.separator();
            ui.label("Note: This injector requires administrator privileges for most processes.");

            self.check_auto_inject();
        });

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

        assert!(logger.is_new(0));

        for _ in 0..NEW_LOG_DURATION_FRAMES {
            logger.update_frame();
        }

        assert!(!logger.is_new(0));
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
