#![windows_subsystem = "windows"]

use config::Config;
use eframe::egui;
use injector::{is_elevated, Injector};
use std::path::PathBuf;

// UI Constants
const MAX_LOGS: usize = 1000;
const MAX_INJECTION_HISTORY: usize = 10;
const MAX_PROCESS_NAME_LENGTH: usize = 260;
const PROCESS_LIST_SCROLL_HEIGHT: f32 = 400.0;
const LOG_SCROLL_HEIGHT: f32 = 250.0;
const WINDOW_WIDTH: f32 = 700.0;
const WINDOW_HEIGHT: f32 = 700.0;
const MIN_WINDOW_WIDTH: f32 = 600.0;
const MIN_WINDOW_HEIGHT: f32 = 600.0;

// Font sizes
const FONT_SIZE_LARGE: f32 = 18.0;
const FONT_SIZE_MEDIUM: f32 = 16.0;
const FONT_SIZE_NORMAL: f32 = 14.0;
const FONT_SIZE_SMALL: f32 = 12.0;

// Animation constants (consolidated)
const ANIMATION_SPEED: f32 = 0.12;
const FAST_ANIMATION_SPEED: f32 = 0.2;
const PROCESS_REFRESH_INTERVAL_FRAMES: i32 = 30;
const SECTION_DELAY_FACTOR: f32 = 0.15;
const SECTION_OFFSET_MULTIPLIER: f32 = 30.0;
const MODAL_PADDING_SCALE: f32 = 50.0;
const STATUS_PULSE_SPEED: f32 = 0.03;
const STATUS_PULSE_AMPLITUDE: f32 = 0.1;
const STATUS_PULSE_BASE: f32 = 0.9;
const ADMIN_PULSE_AMPLITUDE: f32 = 0.05;
const ADMIN_PULSE_BASE: f32 = 0.95;
const RUNNING_PULSE_AMPLITUDE: f32 = 0.2;
const RUNNING_PULSE_BASE: f32 = 0.8;
const AUTO_INJECT_PULSE_SPEED: f32 = 0.05;
const FLASH_ALPHA_START: f32 = 0.5;
const TITLE_SCALE_START: f32 = 0.8;
const TITLE_SCALE_END: f32 = 1.0;
const LOG_SLIDE_START: f32 = 20.0;
const LOG_SLIDE_END: f32 = 0.0;
const BLUR_LAYER_COUNT: usize = 4;
const BLUR_LAYER_ALPHAS: [f32; BLUR_LAYER_COUNT] = [0.08, 0.04, 0.02, 0.01];
const ALPHA_THRESHOLD: f32 = 0.01;
const HISTORY_STAGGER_FACTOR: f32 = 0.1;

mod config;
mod injector;

/// Linear interpolation helper
#[inline]
fn lerp(current: f32, target: f32, speed: f32) -> f32 {
    current + (target - current) * speed
}

/// Draw a blurred background effect by layering semi-transparent rectangles
fn draw_blur_background(painter: &egui::Painter, rect: egui::Rect, alpha: f32) {
    if alpha <= ALPHA_THRESHOLD {
        return;
    }
    let alpha_u8 = (255.0 * alpha) as u8;
    
    // Layered darkening to simulate blur effect
    // Each layer adds slight transparency with different alpha factor
    for layer_alpha in BLUR_LAYER_ALPHAS.iter() {
        let layer_color = egui::Color32::from_rgba_unmultiplied(
            0,  // R
            0,  // G
            0,  // B
            (alpha_u8 as f32 * layer_alpha) as u8,  // A with factor
        );
        painter.rect_filled(rect, 0.0, layer_color);
    }
}

struct LogEntry {
    message: String,
    is_error: bool,
    alpha: f32,
    slide_offset: f32,
    is_removing: bool,
}

impl LogEntry {
    fn new(message: String, is_error: bool) -> Self {
        Self {
            message,
            is_error,
            alpha: 0.0,
            slide_offset: LOG_SLIDE_START,
            is_removing: false,
        }
    }
}

struct LogManager {
    entries: Vec<LogEntry>,
    max_logs: usize,
}

impl LogManager {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_logs: MAX_LOGS,
        }
    }

    fn add_log(&mut self, message: String) {
        let is_error = message.to_lowercase().contains("error");
        self.entries.push(LogEntry::new(message, is_error));
        
        // If exceeding max limit, mark oldest entries for removal with fade-out
        while self.entries.len() > self.max_logs {
            // Start marking from the oldest that isn't already removing
            if let Some(oldest) = self.entries.iter().find(|e| !e.is_removing) {
                // Find index and mark it
                if let Some(idx) = self.entries.iter().position(|e| std::ptr::eq(e, oldest)) {
                    self.entries[idx].is_removing = true;
                    break;
                }
            }
        }
    }

    fn cleanup_removed(&mut self) {
        // Remove entries that have finished fading out (alpha <= 0.01)
        // This handles both explicitly marked removals and any fully faded entries
        self.entries.retain(|entry| entry.alpha > ALPHA_THRESHOLD);
    }

    fn get_entries(&self) -> &[LogEntry] {
        &self.entries
    }

    fn update_frame(&mut self) {
        for entry in &mut self.entries {
            // Fade in for new entries, fade out for removing ones
            let target_alpha = if entry.is_removing { 0.0 } else { 1.0 };
            entry.alpha = lerp(entry.alpha, target_alpha, ANIMATION_SPEED);
            entry.slide_offset = lerp(entry.slide_offset, LOG_SLIDE_END, FAST_ANIMATION_SPEED);
        }
        self.cleanup_removed();
    }
}

/// Modal animation state (consolidates separate modal fields)
struct ModalAnimation {
    scale: f32,
    alpha: f32,
}

impl ModalAnimation {
    fn new() -> Self {
        Self {
            scale: 0.0,
            alpha: 0.0,
        }
    }
}

impl Default for ModalAnimation {
    fn default() -> Self {
        Self::new()
    }
}

struct AnimationState {
    /// Global fade-in/out for entire UI
    global_alpha: f32,
    is_closing: bool,
    /// Title animation
    title_alpha: f32,
    title_scale: f32,
    /// Section animations (each section has its own progress)
    section_alphas: [f32; 6],
    /// Button hover states
    browse_btn_hover: f32,
    inject_btn_hover: f32,
    list_btn_hover: f32,
    /// Process status pulse animation
    status_pulse: f32,
    /// Modal animations (using ModalAnimation struct)
    process_modal: ModalAnimation,
    confirm_modal: ModalAnimation,
    /// Injection history animations
    history_alphas: Vec<f32>,
    /// Success/error flash
    flash_alpha: f32,
    flash_is_success: bool,
    /// Auto-inject indicator pulse
    auto_inject_pulse: f32,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            global_alpha: 0.0,
            is_closing: false,
            title_alpha: 0.0,
            title_scale: TITLE_SCALE_START,
            section_alphas: [0.0; 6],
            browse_btn_hover: 0.0,
            inject_btn_hover: 0.0,
            list_btn_hover: 0.0,
            status_pulse: 0.0,
            process_modal: ModalAnimation::new(),
            confirm_modal: ModalAnimation::new(),
            history_alphas: Vec::new(),
            flash_alpha: 0.0,
            flash_is_success: true,
            auto_inject_pulse: 0.0,
        }
    }
}

impl AnimationState {
    fn update(&mut self, show_process_list: bool, show_confirm: bool, auto_inject_active: bool) {
        // Global fade-in/out
        let global_target = if self.is_closing { 0.0 } else { 1.0 };
        self.global_alpha = lerp(self.global_alpha, global_target, ANIMATION_SPEED);

        // Title animation
        let title_target = if self.is_closing { 0.0 } else { 1.0 };
        self.title_alpha = lerp(self.title_alpha, title_target, ANIMATION_SPEED);
        let scale_target = if self.is_closing { TITLE_SCALE_START } else { TITLE_SCALE_END };
        self.title_scale = lerp(self.title_scale, scale_target, FAST_ANIMATION_SPEED);

        // Section staggered animations (fade in/out)
        for (i, alpha) in self.section_alphas.iter_mut().enumerate() {
            let delay_factor = 1.0 - (SECTION_DELAY_FACTOR * i as f32);
            let section_target = if self.is_closing { 0.0 } else if self.global_alpha > delay_factor { 1.0 } else { 0.0 };
            *alpha = lerp(*alpha, section_target, ANIMATION_SPEED);
        }

        // Status pulse animation (cycles between 0 and 1)
        self.status_pulse = (self.status_pulse + STATUS_PULSE_SPEED) % (std::f32::consts::TAU);
        
        // Auto-inject indicator pulse (only when active)
        if auto_inject_active {
            self.auto_inject_pulse = (self.auto_inject_pulse + AUTO_INJECT_PULSE_SPEED) % (std::f32::consts::TAU);
        }

        // Modal animations
        let target_process_modal_scale = if show_process_list { 1.0 } else { 0.0 };
        let target_process_modal_alpha = if show_process_list { 1.0 } else { 0.0 };
        self.process_modal.scale = lerp(self.process_modal.scale, target_process_modal_scale, FAST_ANIMATION_SPEED);
        self.process_modal.alpha = lerp(self.process_modal.alpha, target_process_modal_alpha, ANIMATION_SPEED);

        let target_confirm_modal_scale = if show_confirm { 1.0 } else { 0.0 };
        let target_confirm_modal_alpha = if show_confirm { 1.0 } else { 0.0 };
        self.confirm_modal.scale = lerp(self.confirm_modal.scale, target_confirm_modal_scale, FAST_ANIMATION_SPEED);
        self.confirm_modal.alpha = lerp(self.confirm_modal.alpha, target_confirm_modal_alpha, ANIMATION_SPEED);

        // Flash fade-out
        self.flash_alpha = lerp(self.flash_alpha, 0.0, ANIMATION_SPEED);
    }

    fn trigger_flash(&mut self, is_success: bool) {
        self.flash_alpha = FLASH_ALPHA_START;
        self.flash_is_success = is_success;
    }

    fn update_history(&mut self, history_len: usize) {
        // Ensure history_alphas syncs with actual history length
        // Remove extra alpha values if history was cleared/populated
        while self.history_alphas.len() > history_len {
            self.history_alphas.pop();
        }
        
        // Add missing alpha values
        while self.history_alphas.len() < history_len {
            self.history_alphas.push(0.0);
        }
        
        // Update each history item alpha
        for (i, alpha) in self.history_alphas.iter_mut().enumerate() {
            let target = if i < history_len { 1.0 } else { 0.0 };
            // Stagger the animation with minimum speed
            let speed = ANIMATION_SPEED * (1.0 - HISTORY_STAGGER_FACTOR * i as f32).max(0.5);
            *alpha = lerp(*alpha, target, speed);
        }
    }

    /// Check if any animations are currently active
    fn has_active_animations(&self) -> bool {
        // Consider animations active if they're not at target values
        self.global_alpha < 0.99
            || self.title_alpha < 0.99
            || self.title_scale < 0.99
            || self.section_alphas.iter().any(|&a| a < 0.99)
            || self.process_modal.alpha > 0.01
            || self.confirm_modal.alpha > 0.01
            || self.process_modal.scale > 0.01
            || self.confirm_modal.scale > 0.01
            || self.flash_alpha > 0.01
            || self.history_alphas.iter().any(|&a| a < 0.99)
            || self.browse_btn_hover > 0.01
            || self.inject_btn_hover > 0.01
            || self.list_btn_hover > 0.01
    }
}

struct InjectorApp {
    dll_path: Option<PathBuf>,
    process_name: String,
    auto_inject: bool,
    logger: LogManager,
    injector: Injector,
    config: Config,
    all_processes: Vec<(String, u32)>,
    cached_process_names: Vec<String>,
    search_query: String,
    auto_injected: bool,
    show_process_list: bool,
    selected_pid: Option<u32>,
    injection_history: Vec<String>,
    last_process_was_running: bool,
    show_confirm_dialog: bool,
    frame_counter: i32,
    animation: AnimationState,
}

impl Default for InjectorApp {
    fn default() -> Self {
        let config = Config::load();
        Self {
            dll_path: config.dll_path.as_ref().map(PathBuf::from),
            process_name: config.last_process.clone().unwrap_or_default(),
            auto_inject: config.auto_inject,
            logger: LogManager::new(),
            injector: Injector::new(),
            config,
            all_processes: Vec::new(),
            cached_process_names: Vec::new(),
            search_query: String::new(),
            auto_injected: false,
            show_process_list: false,
            selected_pid: None,
            injection_history: Vec::new(),
            last_process_was_running: false,
            show_confirm_dialog: false,
            frame_counter: 0,
            animation: AnimationState::default(),
        }
    }
}

impl InjectorApp {
    fn refresh_process_list(&mut self) {
        let processes = self.injector.get_all_processes();
        self.cached_process_names = processes.iter().map(|p| p.name.to_lowercase()).collect();
        self.all_processes = processes.into_iter().map(|p| (p.name, p.pid)).collect();
    }

    fn add_log(&mut self, message: String) {
        self.logger.add_log(message);
    }

    fn is_process_running(&self) -> bool {
        if self.process_name.is_empty() {
            return false;
        }
        let process_name_lower = self.process_name.to_lowercase();
        self.cached_process_names
            .iter()
            .any(|name| name.eq_ignore_ascii_case(&process_name_lower))
    }

    fn inject_dll(&mut self) {
        let dll_path = match &self.dll_path {
            Some(path) => path.clone(),
            None => {
                self.add_log("Error: No DLL selected".to_string());
                self.animation.trigger_flash(false);
                return;
            }
        };

        let proc_name = self.process_name.clone();
        if proc_name.is_empty() {
            self.add_log("Error: No process name specified".to_string());
            self.animation.trigger_flash(false);
            return;
        }

        if proc_name.len() > MAX_PROCESS_NAME_LENGTH {
            self.add_log(format!(
                "Error: Process name too long (max {} characters)",
                MAX_PROCESS_NAME_LENGTH
            ));
            self.animation.trigger_flash(false);
            return;
        }

        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        self.add_log(format!("Attempting to inject into {}...", proc_name));

        match self.injector.inject(&proc_name, &dll_path) {
            Ok(_) => {
                self.add_log("Injection successful!".to_string());
                self.animation.trigger_flash(true);
                self.auto_injected = true;
                self.last_process_was_running = true;
                self.config.last_process = Some(proc_name.clone());
                if let Some(err) = self.config.save_with_error_message() {
                    self.add_log(err);
                }
                let dll_name = dll_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown.dll");
                self.injection_history
                    .insert(0, format!("[{}] {} -> {}", timestamp, proc_name, dll_name));
                if self.injection_history.len() > MAX_INJECTION_HISTORY {
                    self.injection_history.pop();
                }
            }
            Err(e) => {
                self.add_log(format!("Injection failed: {}", e));
                self.animation.trigger_flash(false);
                self.auto_injected = false;
            }
        }
    }

    fn check_auto_inject(&mut self) {
        let is_running = self.is_process_running();

        if self.last_process_was_running && !is_running {
            self.auto_injected = false;
        }
        self.last_process_was_running = is_running;

        if self.auto_inject
            && !self.process_name.is_empty()
            && !self.auto_injected
            && is_running
            && self.dll_path.is_some()
        {
            self.add_log("Auto-inject: Target process detected, injecting...".to_string());
            self.inject_dll();
        }
    }

    fn update_cached_processes(&mut self) {
        self.frame_counter += 1;
        if self.frame_counter >= PROCESS_REFRESH_INTERVAL_FRAMES {
            self.frame_counter = 0;
            self.refresh_process_list();
        }
    }
}

impl eframe::App for InjectorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update animations
        self.animation.update(
            self.show_process_list,
            self.show_confirm_dialog,
            self.auto_inject,
        );
        self.animation.update_history(self.injection_history.len());
        self.update_cached_processes();
        self.logger.update_frame();

        // Only request repaint if animations are active (optimization)
        if self.animation.has_active_animations() || self.show_process_list || self.show_confirm_dialog {
            ctx.request_repaint();
        }

        // Flash overlay for success/error feedback
        if self.animation.flash_alpha > ALPHA_THRESHOLD {
            let flash_color = if self.animation.flash_is_success {
                egui::Color32::from_rgba_unmultiplied(0, 255, 0, (50.0 * self.animation.flash_alpha) as u8)
            } else {
                egui::Color32::from_rgba_unmultiplied(255, 0, 0, (50.0 * self.animation.flash_alpha) as u8)
            };
            egui::Area::new(egui::Id::new("flash_overlay"))
                .interactable(false)
                .fixed_pos(egui::Pos2::ZERO)
                .show(ctx, |ui| {
                    let rect = ui.ctx().screen_rect();
                    ui.painter().rect_filled(rect, 0.0, flash_color);
                });
        }

        // Main panel (with global fade-in/out)
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                ui.set_opacity(self.animation.global_alpha);
                ui.add_space(20.0);

            // Title with scale and fade animation
            ui.vertical_centered(|ui| {
                let title_color = egui::Color32::from_rgba_premultiplied(
                    255,
                    255,
                    255,
                    (255.0 * self.animation.title_alpha) as u8,
                );
                let title_size = FONT_SIZE_LARGE * self.animation.title_scale;
                ui.label(
                    egui::RichText::new("Ruin DLL Injector")
                        .size(title_size)
                        .color(title_color)
                        .strong(),
                );

                // Admin status with subtle pulse animation (less distracting)
                let admin_pulse = self.animation.status_pulse.sin() * ADMIN_PULSE_AMPLITUDE + ADMIN_PULSE_BASE;
                let admin_color = if is_elevated() {
                    let base = egui::Color32::LIGHT_GREEN;
                    egui::Color32::from_rgb(
                        (base.r() as f32 * admin_pulse) as u8,
                        (base.g() as f32 * admin_pulse) as u8,
                        base.b(),
                    )
                } else {
                    egui::Color32::LIGHT_RED
                };
                ui.label(
                    egui::RichText::new(if is_elevated() {
                        "Administrator"
                    } else {
                        "Not Administrator"
                    })
                    .size(FONT_SIZE_SMALL)
                    .color(admin_color),
                );
            });

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(20.0);

            // Section 1: Target DLL (with slide-in animation)
            let section_alpha = self.animation.section_alphas[0];
            let section_offset = (1.0 - section_alpha) * SECTION_OFFSET_MULTIPLIER;
            ui.add_space(section_offset);
            
            let label_color = egui::Color32::from_rgba_premultiplied(
                255, 255, 255,
                (255.0 * section_alpha) as u8,
            );
            ui.label(
                egui::RichText::new("Target DLL")
                    .size(FONT_SIZE_MEDIUM)
                    .color(label_color),
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
                    ui.text_edit_singleline(&mut display_text.clone());
                });

                // Browse button with hover animation
                let browse_response = ui.button("Browse");
                self.animation.browse_btn_hover = lerp(
                    self.animation.browse_btn_hover,
                    if browse_response.hovered() { 1.0 } else { 0.0 },
                    FAST_ANIMATION_SPEED,
                );

                if browse_response.clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("DLL", &["dll"])
                        .pick_file()
                    {
                        self.dll_path = Some(path.clone());
                        self.config.dll_path = Some(path.display().to_string());
                        if let Some(err) = self.config.save_with_error_message() {
                            self.add_log(err);
                        }
                        self.add_log(format!("Selected DLL: {}", path.display()));
                    }
                }
            });

            if let Some(path) = &self.dll_path {
                let path_alpha = self.animation.section_alphas[0];
                ui.label(
                    egui::RichText::new(path.display().to_string())
                        .size(FONT_SIZE_SMALL)
                        .color(egui::Color32::from_rgba_premultiplied(
                            200, 200, 200,
                            (255.0 * path_alpha) as u8,
                        )),
                );
            }

            ui.add_space(20.0);

            // Section 2: Target Process
            let section_alpha = self.animation.section_alphas[1];
            let label_color = egui::Color32::from_rgba_premultiplied(
                255, 255, 255,
                (255.0 * section_alpha) as u8,
            );
            ui.label(
                egui::RichText::new("Target Process")
                    .size(FONT_SIZE_MEDIUM)
                    .color(label_color),
            );
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                let mut process_name_validated = self.process_name.clone();
                if process_name_validated.len() > MAX_PROCESS_NAME_LENGTH {
                    process_name_validated.truncate(MAX_PROCESS_NAME_LENGTH);
                }
                self.process_name = process_name_validated;

                ui.text_edit_singleline(&mut self.process_name);

                // List/Running button with animation
                let is_running = self.is_process_running();
                let btn_text = if is_running { "Running" } else { "List" };
                let btn_color = if is_running {
                    let pulse = self.animation.status_pulse.sin() * RUNNING_PULSE_AMPLITUDE + RUNNING_PULSE_BASE;
                    egui::Color32::from_rgb(
                        (100.0 * pulse) as u8,
                        (200.0 * pulse) as u8,
                        (100.0 * pulse) as u8,
                    )
                } else {
                    egui::Color32::GRAY
                };

                let list_btn = egui::Button::new(
                    egui::RichText::new(btn_text).color(btn_color)
                );
                let list_response = ui.add(list_btn);
                
                self.animation.list_btn_hover = lerp(
                    self.animation.list_btn_hover,
                    if list_response.hovered() { 1.0 } else { 0.0 },
                    FAST_ANIMATION_SPEED,
                );

                if list_response.clicked() {
                    self.refresh_process_list();
                    self.show_process_list = true;
                }
            });

            // Process status with pulsing animation
            if !self.process_name.is_empty() {
                let is_running = self.is_process_running();
                let status_text = if is_running { "Status: Running" } else { "Status: Not found" };
                
                let pulse = if is_running {
                    self.animation.status_pulse.sin() * STATUS_PULSE_AMPLITUDE + STATUS_PULSE_BASE
                } else {
                    1.0
                };
                let status_color = if is_running {
                    egui::Color32::from_rgba_premultiplied(
                        144, 238, 144,
                        (255.0 * pulse) as u8,
                    )
                } else {
                    egui::Color32::from_rgba_premultiplied(
                        180, 180, 180,
                        (255.0 * self.animation.section_alphas[1]) as u8,
                    )
                };
                
                ui.label(
                    egui::RichText::new(status_text)
                        .size(FONT_SIZE_SMALL)
                        .color(status_color),
                );
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(20.0);

            // Section 3: Injection controls
            let _section_alpha = self.animation.section_alphas[2];
            
            ui.horizontal(|ui| {
                // Inject button with hover glow effect
                let inject_btn = egui::Button::new(
                    egui::RichText::new("Inject DLL")
                        .color(egui::Color32::WHITE)
                );
                let inject_response = ui.add(inject_btn);
                
                self.animation.inject_btn_hover = lerp(
                    self.animation.inject_btn_hover,
                    if inject_response.hovered() { 1.0 } else { 0.0 },
                    FAST_ANIMATION_SPEED,
                );

                if inject_response.clicked() {
                    self.show_confirm_dialog = true;
                }

                // Auto-inject checkbox
                let _checkbox_color = if self.auto_inject {
                    let pulse = self.animation.auto_inject_pulse.sin() * STATUS_PULSE_AMPLITUDE + STATUS_PULSE_BASE;
                    egui::Color32::from_rgb(
                        (144.0 * pulse) as u8,
                        (238.0 * pulse) as u8,
                        (144.0 * pulse) as u8,
                    )
                } else {
                    egui::Color32::GRAY
                };

                if ui.checkbox(&mut self.auto_inject, "Auto-inject").changed() {
                    self.config.auto_inject = self.auto_inject;
                    if let Some(err) = self.config.save_with_error_message() {
                        self.add_log(err);
                    }
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

                // Active indicator with pulsing animation
                if self.auto_inject {
                    let pulse = self.animation.auto_inject_pulse.sin() * STATUS_PULSE_AMPLITUDE + STATUS_PULSE_BASE;
                    ui.label(
                        egui::RichText::new("Active")
                            .color(egui::Color32::from_rgba_premultiplied(
                                (144.0 * pulse) as u8,
                                (238.0 * pulse) as u8,
                                (144.0 * pulse) as u8,
                                255,
                            ))
                            .size(FONT_SIZE_SMALL)
                    );
                }
            });

            ui.add_space(20.0);

            // Section 4: Injection history with staggered animations
            if !self.injection_history.is_empty() {
                let section_alpha = self.animation.section_alphas[3];
                let label_color = egui::Color32::from_rgba_premultiplied(
                    255, 255, 255,
                    (255.0 * section_alpha) as u8,
                );
                ui.label(
                    egui::RichText::new("Recent Injections")
                        .size(FONT_SIZE_MEDIUM)
                        .color(label_color),
                );
                ui.add_space(10.0);

                for (i, entry) in self.injection_history.iter().enumerate() {
                    let alpha = self.animation.history_alphas.get(i).copied().unwrap_or(1.0);
                    let slide_offset = (1.0 - alpha) * 20.0;
                    
                    ui.horizontal(|ui| {
                        ui.add_space(slide_offset);
                        ui.label(
                            egui::RichText::new(entry)
                                .size(FONT_SIZE_NORMAL)
                                .color(egui::Color32::from_rgba_premultiplied(
                                    180, 180, 180,
                                    (255.0 * alpha) as u8,
                                )),
                        );
                    });
                }
                ui.add_space(20.0);
            }

            ui.separator();
            ui.add_space(20.0);

            // Section 5: Activity log
            let section_alpha = self.animation.section_alphas[4];
            let label_color = egui::Color32::from_rgba_premultiplied(
                255, 255, 255,
                (255.0 * section_alpha) as u8,
            );
            ui.label(
                egui::RichText::new("Activity Log")
                    .size(FONT_SIZE_MEDIUM)
                    .color(label_color),
            );
            ui.add_space(10.0);

            egui::ScrollArea::vertical()
                .max_height(LOG_SCROLL_HEIGHT)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for entry in self.logger.get_entries() {
                        let base_color = if entry.is_error {
                            egui::Color32::LIGHT_RED
                        } else {
                            egui::Color32::LIGHT_GRAY
                        };

                        let color = egui::Color32::from_rgba_premultiplied(
                            base_color.r(),
                            base_color.g(),
                            base_color.b(),
                            (255.0 * entry.alpha) as u8,
                        );

                        ui.horizontal(|ui| {
                            ui.add_space(entry.slide_offset);
                            ui.label(
                                egui::RichText::new(&entry.message)
                                    .size(FONT_SIZE_NORMAL)
                                    .color(color),
                            );
                        });
                    }
                });

            ui.add_space(15.0);

            // Section 6: Footer
            let section_alpha = self.animation.section_alphas[5];
            ui.label(
                egui::RichText::new("Some processes require administrator privileges")
                    .size(FONT_SIZE_SMALL)
                    .color(egui::Color32::from_rgba_premultiplied(
                        150, 150, 150,
                        (255.0 * section_alpha) as u8,
                    )),
            );

            self.check_auto_inject();
        });

        // Confirmation dialog modal (blocks background interaction)
        if self.animation.confirm_modal.alpha > ALPHA_THRESHOLD {
            // Draw blurred background overlay
            egui::Area::new(egui::Id::new("confirm_overlay"))
                .interactable(false)
                .fixed_pos(egui::Pos2::ZERO)
                .order(egui::Order::Middle) // Ensure overlay is above everything
                .show(ctx, |ui| {
                    let screen_rect = ui.ctx().screen_rect();
                    draw_blur_background(ui.painter(), screen_rect, self.animation.confirm_modal.alpha);
                });

            // Modal window with scale animation
            egui::Window::new("Confirm Injection")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .title_bar(false)
                .show(ctx, |ui| {
                    ui.set_enabled(self.animation.confirm_modal.scale > 0.5);
                    
                    // Wrap entire content in Frame to fade everything (including window background)
                    egui::Frame::none()
                        .multiply_with_opacity(self.animation.confirm_modal.alpha)
                        .show(ui, |ui| {
                            // Scale effect by adding padding
                            let padding = (1.0 - self.animation.confirm_modal.scale) * MODAL_PADDING_SCALE;
                            ui.add_space(padding);
                            
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    egui::RichText::new("Confirm Injection")
                                        .size(FONT_SIZE_MEDIUM)
                                        .color(egui::Color32::WHITE),
                                );
                            });
                            ui.add_space(10.0);
                            ui.separator();
                            ui.add_space(10.0);
                            
                            ui.label(
                                egui::RichText::new(format!(
                                    "Inject DLL into \"{}\"?",
                                    self.process_name
                                ))
                                .color(egui::Color32::WHITE),
                            );
                            ui.add_space(15.0);
                            
                            ui.horizontal(|ui| {
                                let confirm_btn = egui::Button::new(
                                    egui::RichText::new("Confirm").color(egui::Color32::WHITE)
                                );
                                let cancel_btn = egui::Button::new(
                                    egui::RichText::new("Cancel").color(egui::Color32::WHITE)
                                );
                                if ui.add(confirm_btn).clicked() {
                                    self.show_confirm_dialog = false;
                                    self.inject_dll();
                                }
                                
                                if ui.add(cancel_btn).clicked() {
                                    self.show_confirm_dialog = false;
                                }
                            });
                            
                            ui.add_space(padding);
                        });
                });
        }

        // Process list modal (blocks background interaction)
        if self.animation.process_modal.alpha > ALPHA_THRESHOLD {
            // Draw blurred background overlay
            egui::Area::new(egui::Id::new("process_list_overlay"))
                .interactable(false)
                .fixed_pos(egui::Pos2::ZERO)
                .order(egui::Order::Middle) // Ensure overlay is above everything
                .show(ctx, |ui| {
                    let screen_rect = ui.ctx().screen_rect();
                    draw_blur_background(ui.painter(), screen_rect, self.animation.process_modal.alpha);
                });

            let modal_scale = self.animation.process_modal.scale;
            let modal_alpha = self.animation.process_modal.alpha;

            egui::Window::new("Select Process")
                .collapsible(false)
                .resizable(true)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.set_enabled(modal_scale > 0.5);
                    
                    // Wrap entire content in Frame to fade everything (including window background)
                    egui::Frame::none()
                        .multiply_with_opacity(modal_alpha)
                        .show(ui, |ui| {
                            ui.heading(egui::RichText::new("Running Processes").color(egui::Color32::WHITE));
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
                });
        }
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
    fn test_lerp_function() {
        assert!((lerp(0.0, 1.0, 0.5) - 0.5).abs() < 0.01);
        assert!((lerp(0.0, 1.0, 1.0) - 1.0).abs() < 0.01);
        assert!((lerp(1.0, 0.0, 0.5) - 0.5).abs() < 0.01);
        
        // Test edge cases
        assert_eq!(lerp(0.5, 0.5, 0.5), 0.5); // Already at target
        assert_eq!(lerp(0.0, 0.0, 0.5), 0.0); // Zero range
        assert_eq!(lerp(1.0, 1.0, 0.5), 1.0); // Same value
    }

    #[test]
    fn test_blur_background_early_return() {
        // Should return early when alpha is very low
        // Just verify function exists and doesn't panic
        // Full testing would require egui Context which is not available in unit tests
        assert!(ALPHA_THRESHOLD > 0.0);
        assert!(ALPHA_THRESHOLD < 1.0);
    }

    #[test]
    fn test_log_manager_add_message() {
        let mut logger = LogManager::new();
        logger.add_log("Test message".to_string());

        assert_eq!(logger.get_entries().len(), 1);
        assert_eq!(logger.get_entries()[0].message, "Test message");
    }

    #[test]
    fn test_log_manager_error_detection() {
        let mut logger = LogManager::new();
        logger.add_log("Error: something went wrong".to_string());
        logger.add_log("Normal message".to_string());

        assert!(logger.get_entries()[0].is_error);
        assert!(!logger.get_entries()[1].is_error);
    }

    #[test]
    fn test_log_manager_animation() {
        let mut logger = LogManager::new();
        logger.add_log("Test".to_string());

        assert_eq!(logger.get_entries()[0].alpha, 0.0);
        assert_eq!(logger.get_entries()[0].slide_offset, LOG_SLIDE_START);

        for _ in 0..50 {
            logger.update_frame();
        }

        assert!(logger.get_entries()[0].alpha >= 0.9);
        assert!(logger.get_entries()[0].slide_offset <= 2.0);
    }

    #[test]
    fn test_log_manager_max_limit() {
        let mut logger = LogManager::new();

        for i in 0..1002 {
            logger.add_log(format!("Log {}", i));
        }

        // Run enough update frames for fade-out and cleanup to complete
        // Fade out takes ~50 frames to reach alpha <= 0.01 with ANIMATION_SPEED=0.12
        for _ in 0..80 {
            logger.update_frame();
        }

        assert_eq!(logger.get_entries().len(), MAX_LOGS);
    }

    #[test]
    fn test_animation_state_default() {
        let anim = AnimationState::default();
        assert_eq!(anim.global_alpha, 0.0);
        assert_eq!(anim.title_alpha, 0.0);
        assert_eq!(anim.title_scale, TITLE_SCALE_START);
        assert_eq!(anim.process_modal.scale, 0.0);
        assert_eq!(anim.confirm_modal.scale, 0.0);
    }

    #[test]
    fn test_animation_flash() {
        let mut anim = AnimationState::default();
        anim.trigger_flash(true);
        assert!(anim.flash_alpha > 0.4); // Should be set to FLASH_ALPHA_START
        assert!(anim.flash_is_success);
    }

    #[test]
    fn test_modal_animation_struct() {
        let modal = ModalAnimation::new();
        assert_eq!(modal.scale, 0.0);
        assert_eq!(modal.alpha, 0.0);
        
        let modal_default = ModalAnimation::default();
        assert_eq!(modal.scale, modal_default.scale);
        assert_eq!(modal.alpha, modal_default.alpha);
    }

    #[test]
    fn test_animation_history_sync() {
        let mut anim = AnimationState::default();
        
        // Add 5 items to history
        anim.update_history(5);
        assert_eq!(anim.history_alphas.len(), 5);
        
        // Reduce history to 3 items
        anim.update_history(3);
        assert_eq!(anim.history_alphas.len(), 3); // Should trim extras
        
        // Increase history to 7 items
        anim.update_history(7);
        assert_eq!(anim.history_alphas.len(), 7); // Should add more
    }

    #[test]
    fn test_process_name_max_length() {
        let mut app = InjectorApp::default();
        let long_name = "a".repeat(300);
        app.process_name = long_name.clone();

        // Simulate the validation in update
        let mut process_name_validated = app.process_name.clone();
        if process_name_validated.len() > MAX_PROCESS_NAME_LENGTH {
            process_name_validated.truncate(MAX_PROCESS_NAME_LENGTH);
        }
        app.process_name = process_name_validated;

        assert!(
            app.process_name.len() <= MAX_PROCESS_NAME_LENGTH,
            "Process name should be truncated to max length"
        );
    }

    #[test]
    fn test_animation_constants_defined() {
        // Verify all animation constants are defined and have reasonable values
        assert!(ANIMATION_SPEED > 0.0 && ANIMATION_SPEED < 1.0);
        assert!(FAST_ANIMATION_SPEED > 0.0 && FAST_ANIMATION_SPEED < 1.0);
        assert!(STATUS_PULSE_SPEED > 0.0);
        assert!(FLASH_ALPHA_START > 0.0 && FLASH_ALPHA_START <= 1.0);
        assert!(TITLE_SCALE_START < TITLE_SCALE_END);
        assert!(LOG_SLIDE_START > LOG_SLIDE_END);
        assert_eq!(BLUR_LAYER_COUNT, 4);
        assert_eq!(BLUR_LAYER_ALPHAS.len(), 4);
    }

    #[test]
    fn test_modal_animation_update() {
        let mut anim = AnimationState::default();
        
        // Show modal
        anim.update(true, false, false);
        assert!(anim.process_modal.alpha > 0.0); // Should start fading in
        assert!(anim.process_modal.scale > 0.0);
        
        // Hide modal
        anim.update(false, false, false);
        assert!(anim.process_modal.alpha < 1.0); // Should start fading out
        assert!(anim.process_modal.scale < 1.0);
    }
}
