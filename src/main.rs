#![windows_subsystem = "windows"]

use eframe::egui;
use std::sync::Arc;
use std::path::PathBuf;
use injector::Injector;
use config::Config;

mod injector;
mod uwp;
mod config;

struct InjectorApp {
    dll_path: Option<PathBuf>,
    process_name: String,
    auto_inject: bool,
    log_messages: Vec<String>,
    injector: Arc<Injector>,
    config: Config,
    injection_enabled: bool,
    show_process_list: bool,
    all_processes: Vec<(String, u32)>,
    search_query: String,
    new_log_start_index: usize,
    new_log_frame_counter: usize,
    error_log_indices: Vec<usize>,
}

impl Default for InjectorApp {
    fn default() -> Self {
        Self {
            dll_path: None,
            process_name: String::new(),
            auto_inject: false,
            log_messages: Vec::new(),
            injector: Arc::new(Injector::new()),
            config: Config::load(),
            injection_enabled: true,
            show_process_list: false,
            all_processes: Vec::new(),
            search_query: String::new(),
            new_log_start_index: 0,
            new_log_frame_counter: 0,
            error_log_indices: Vec::new(),
        }
    }
}

impl InjectorApp {
    fn refresh_process_list(&mut self) {
        let processes = self.injector.get_all_processes();
        self.all_processes = processes.into_iter().map(|p| (p.name, p.pid)).collect();
    }

    fn add_log(&mut self, message: String) {
        self.new_log_start_index = self.log_messages.len();
        self.new_log_frame_counter = 0;
        
        let is_error = message.to_lowercase().contains("error");
        self.log_messages.push(message);
        
        if is_error {
            self.error_log_indices.push(self.log_messages.len() - 1);
        }
        
        if self.log_messages.len() > 1000 {
            self.log_messages.remove(0);
            self.new_log_start_index = self.new_log_start_index.saturating_sub(1);
            self.error_log_indices = self.error_log_indices.iter()
                .filter_map(|&i| i.checked_sub(1))
                .collect();
        }
    }

    fn inject_dll(&mut self) {
        if !self.injection_enabled {
            self.add_log("Injection is disabled".to_string());
            return;
        }

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
            }
            Err(e) => {
                self.add_log(format!("Injection failed: {}", e));
            }
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
                    self.show_process_list = true;
                }
            });

            if self.show_process_list {
                egui::Window::new("Select Process")
                    .collapsible(false)
                    .resizable(true)
                    .show(ctx, |ui| {
                        ui.heading("Running Processes");
                        ui.separator();
                        
                        ui.horizontal(|ui| {
                            ui.label("🔍 Search:");
                            ui.text_edit_singleline(&mut self.search_query);
                        });
                        
                        ui.separator();
                        
                        let search_lower = self.search_query.to_lowercase();
                        let mut matched_process: Option<(String, u32)> = None;
                        
                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .show(ui, |ui| {
                                let mut has_matches = false;
                                for (name, pid) in &self.all_processes {
                                    let name_lower = name.to_lowercase();
                                    if search_lower.is_empty() || name_lower.contains(&search_lower) {
                                        has_matches = true;
                                        if ui.button(format!("Select: {} (PID: {})", name, pid)).clicked() {
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
                            self.show_process_list = false;
                            self.search_query.clear();
                        }
                        
                        if let Some((name, _)) = matched_process {
                            self.process_name = name.clone();
                            self.add_log(format!("Selected process: {}", name));
                            self.show_process_list = false;
                            self.search_query.clear();
                        }
                    });
            }

            ui.horizontal(|ui| {
                if ui.button("Inject").clicked() {
                    self.inject_dll();
                }

                ui.checkbox(&mut self.auto_inject, "Auto Inject");
            });

            ui.separator();

            ui.heading("Logs");
            egui::ScrollArea::vertical()
                .max_height(150.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for (i, log) in self.log_messages.iter().enumerate() {
                        let is_new = i >= self.new_log_start_index && self.new_log_frame_counter < 120;
                        let is_error = self.error_log_indices.contains(&i);
                        
                        let color = if is_error {
                            egui::Color32::RED
                        } else if is_new {
                            egui::Color32::from_rgb(100, 255, 100)
                        } else {
                            egui::Color32::WHITE
                        };
                        
                        ui.colored_label(color, log);
                    }
                    
                    if self.new_log_frame_counter < 120 {
                        self.new_log_frame_counter += 1;
                    }
                });

            ui.separator();
            ui.label("Note: This injector requires administrator privileges to inject into UWP processes.");
            ui.label("The DLL will automatically be granted 'All Applications Packages' permission.");
        });

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 480.0])
            .with_min_inner_size([350.0, 400.0])
            .with_title("Ruin DLL Injector"),
        ..Default::default()
    };

    eframe::run_native(
        "Ruin DLL Injector",
        options,
        Box::new(|_cc| Box::<InjectorApp>::default()),
    )
}
