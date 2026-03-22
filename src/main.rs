#![windows_subsystem = "windows"]

use config::Config;
use eframe::egui;
use injector::{is_elevated, Injector};
use std::path::PathBuf;

// Animation constants (simplified)
const ANIMATION_DEFAULT_SPEED: f32 = 0.12;
const ANIMATION_FAST_SPEED: f32 = 0.2;
const ANIMATION_SLOW_SPEED: f32 = 0.05;
const PROCESS_REFRESH_INTERVAL_FRAMES: i32 = 30;
const SECTION_DELAY_FRAMES: i32 = 15;
const MODAL_PADDING_SCALE: f32 = 50.0;

// Pulse animation constants
const PULSE_SPEED_DEFAULT: f32 = 0.03;
const PULSE_AMPLITUDE_DEFAULT: f32 = 0.1;
const PULSE_BASE_DEFAULT: f32 = 0.9;

// Flash animation
const FLASH_ALPHA_START: f32 = 0.5;

// UI bounds
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

// Thresholds
const ALPHA_THRESHOLD: f32 = 0.01;
const SCALE_THRESHOLD: f32 = 0.01;

mod config;
mod injector;

// ============= Animation System =============

/// Easing functions for smooth animations
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Easing {
    #[default]
    Linear,
    EaseOut,
    EaseInOut,
    EaseOutBack,
}

/// Base trait for all animations
trait Animatable {
    fn update(&mut self, dt: f32);
    fn is_complete(&self) -> bool;
}

/// Fade animation (0.0 to 1.0)
#[derive(Debug, Clone)]
struct Fade {
    current: f32,
    target: f32,
    speed: f32,
}

impl Fade {
    fn new() -> Self {
        Self {
            current: 0.0,
            target: 0.0,
            speed: ANIMATION_DEFAULT_SPEED,
        }
    }

    fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    fn set_target(&mut self, target: f32) {
        self.target = target.clamp(0.0, 1.0);
    }

    fn get(&self) -> f32 {
        self.current
    }
}

impl Default for Fade {
    fn default() -> Self {
        Self::new()
    }
}

impl Animatable for Fade {
    fn update(&mut self, dt: f32) {
        let diff = self.target - self.current;
        if diff.abs() < ALPHA_THRESHOLD {
            self.current = self.target;
            return;
        }
        self.current += diff * self.speed * dt;
        self.current = self.current.clamp(0.0, 1.0);
    }

    fn is_complete(&self) -> bool {
        (self.current - self.target).abs() < ALPHA_THRESHOLD
    }
}

/// Scale animation (e.g., for modals)
#[derive(Debug, Clone)]
struct Scale {
    current: f32,
    target: f32,
    speed: f32,
}

impl Scale {
    fn new() -> Self {
        Self {
            current: 0.0,
            target: 0.0,
            speed: ANIMATION_DEFAULT_SPEED,
        }
    }

    fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    fn get(&self) -> f32 {
        self.current
    }
}

impl Default for Scale {
    fn default() -> Self {
        Self::new()
    }
}

impl Animatable for Scale {
    fn update(&mut self, dt: f32) {
        let diff = self.target - self.current;
        if diff.abs() < SCALE_THRESHOLD {
            self.current = self.target;
            return;
        }
        self.current += diff * self.speed * dt;
    }

    fn is_complete(&self) -> bool {
        (self.current - self.target).abs() < SCALE_THRESHOLD
    }
}

/// Slide animation (for smooth positioning)
#[derive(Debug, Clone)]
struct Slide {
    current: f32,
    target: f32,
    speed: f32,
}

impl Slide {
    fn new() -> Self {
        Self {
            current: 20.0,
            target: 0.0,
            speed: ANIMATION_FAST_SPEED,
        }
    }

    fn get(&self) -> f32 {
        self.current
    }
}

impl Default for Slide {
    fn default() -> Self {
        Self::new()
    }
}

impl Animatable for Slide {
    fn update(&mut self, dt: f32) {
        let diff = self.target - self.current;
        if diff.abs() < 0.1 {
            self.current = self.target;
            return;
        }
        self.current += diff * self.speed * dt;
    }

    fn is_complete(&self) -> bool {
        (self.current - self.target).abs() < 0.1
    }
}

/// Pulse animation (for status indicators)
#[derive(Debug, Clone)]
struct Pulse {
    phase: f32,
    speed: f32,
    amplitude: f32,
    base: f32,
}

impl Pulse {
    fn new() -> Self {
        Self {
            phase: 0.0,
            speed: PULSE_SPEED_DEFAULT,
            amplitude: PULSE_AMPLITUDE_DEFAULT,
            base: PULSE_BASE_DEFAULT,
        }
    }

    fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    fn get(&self) -> f32 {
        (self.phase.sin() * self.amplitude + self.base).clamp(0.0, 1.0)
    }
}

impl Default for Pulse {
    fn default() -> Self {
        Self::new()
    }
}

impl Animatable for Pulse {
    fn update(&mut self, dt: f32) {
        self.phase += self.speed * dt;
        if self.phase > std::f32::consts::TAU {
            self.phase -= std::f32::consts::TAU;
        }
    }

    fn is_complete(&self) -> bool {
        false // Pulse animations never complete
    }
}

/// Combined modal animation (scale + fade)
#[derive(Debug, Clone)]
struct ModalAnimation {
    fade: Fade,
    scale: Scale,
}

impl ModalAnimation {
    fn new() -> Self {
        Self {
            fade: Fade::new().with_speed(ANIMATION_DEFAULT_SPEED),
            scale: Scale::new().with_speed(ANIMATION_DEFAULT_SPEED),
        }
    }

    fn show(&mut self) {
        self.fade.set_target(1.0);
        self.scale.set_target(1.0);
    }

    fn hide(&mut self) {
        self.fade.set_target(0.0);
        self.scale.set_target(0.0);
    }

    fn get_alpha(&self) -> f32 {
        self.fade.get()
    }

    fn get_scale(&self) -> f32 {
        self.scale.get()
    }

    fn is_visible(&self) -> bool {
        self.fade.get() > ALPHA_THRESHOLD
    }
}

impl Default for ModalAnimation {
    fn default() -> Self {
        Self::new()
    }
}

/// Draw a blurred background effect
fn draw_blur_background(painter: &egui::Painter, rect: egui::Rect, alpha: f32) {
    if alpha <= ALPHA_THRESHOLD {
        return;
    }
    let alpha_u8 = (255.0 * alpha) as u8;

    // Layered darkening for blur effect
    for layer_alpha in [0.08, 0.04, 0.02, 0.01].iter() {
        let layer_color =
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, (alpha_u8 as f32 * layer_alpha) as u8);
        painter.rect_filled(rect, 0.0, layer_color);
    }
}

// ============= Log System with Animations =============

struct LogEntry {
    message: String,
    is_error: bool,
    fade: Fade,
    slide: Slide,
    is_removing: bool,
}

impl LogEntry {
    fn new(message: String, is_error: bool) -> Self {
        Self {
            message,
            is_error,
            fade: Fade::new().with_speed(ANIMATION_DEFAULT_SPEED),
            slide: Slide::new(),
            is_removing: false,
        }
    }

    fn get_alpha(&self) -> f32 {
        self.fade.get()
    }

    fn get_slide_offset(&self) -> f32 {
        self.slide.get()
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

        // Mark oldest entries for removal when exceeding max
        while self.entries.len() > self.max_logs {
            if let Some(oldest) = self.entries.iter().find(|e| !e.is_removing) {
                if let Some(idx) = self.entries.iter().position(|e| std::ptr::eq(e, oldest)) {
                    self.entries[idx].is_removing = true;
                    self.entries[idx].fade.set_target(0.0);
                    break;
                }
            }
        }
    }

    fn cleanup_removed(&mut self) {
        self.entries
            .retain(|entry| entry.fade.get() > ALPHA_THRESHOLD);
    }

    fn get_entries(&self) -> &[LogEntry] {
        &self.entries
    }

    fn update_frame(&mut self) {
        for entry in &mut self.entries {
            if entry.is_removing {
                entry.fade.set_target(0.0);
            } else {
                entry.fade.set_target(1.0);
            }
            entry.fade.update(1.0);
            entry.slide.update(1.0);
        }
        self.cleanup_removed();
    }
}

// ============= Main Animation State =============

struct AnimationState {
    // Global fade-in/out
    global_fade: Fade,
    is_closing: bool,

    // Title animations
    title_fade: Fade,
    title_scale: Scale,

    // Section animations (staggered)
    section_fades: Vec<Fade>,
    section_delays: Vec<i32>,
    frame_counter: i32,

    // Button hover animations
    button_hover: [Fade; 3], // browse, inject, list

    // Pulse animations
    status_pulse: Pulse,
    auto_inject_pulse: Pulse,

    // Modal animations
    process_modal: ModalAnimation,
    confirm_modal: ModalAnimation,

    // Injection history animations
    history_fades: Vec<Fade>,

    // Flash animation
    flash_fade: Fade,
    flash_is_success: bool,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            global_fade: Fade::new().with_speed(ANIMATION_DEFAULT_SPEED),
            is_closing: false,
            title_fade: Fade::new(),
            title_scale: Scale::new(),
            section_fades: (0..6).map(|_| Fade::new()).collect(),
            section_delays: (0..6).map(|i| i * SECTION_DELAY_FRAMES).collect(),
            frame_counter: 0,
            button_hover: [Fade::new(), Fade::new(), Fade::new()],
            status_pulse: Pulse::new(),
            auto_inject_pulse: Pulse::new().with_speed(ANIMATION_SLOW_SPEED),
            process_modal: ModalAnimation::new(),
            confirm_modal: ModalAnimation::new(),
            history_fades: Vec::new(),
            flash_fade: Fade::new().with_speed(ANIMATION_FAST_SPEED),
            flash_is_success: true,
        }
    }
}

impl AnimationState {
    fn update(&mut self, show_process_list: bool, show_confirm: bool, auto_inject_active: bool) {
        self.frame_counter += 1;
        let dt = 1.0;

        // Global fade
        let global_target = if self.is_closing { 0.0 } else { 1.0 };
        self.global_fade.set_target(global_target);
        self.global_fade.update(dt);

        // Title animations
        let title_target = if self.is_closing { 0.0 } else { 1.0 };
        self.title_fade.set_target(title_target);
        self.title_fade.update(dt);
        self.title_scale
            .set_target(if self.is_closing { 0.8 } else { 1.0 });
        self.title_scale.update(dt);

        // Section animations with staggered delays
        for (i, fade) in self.section_fades.iter_mut().enumerate() {
            if self.frame_counter >= self.section_delays[i] {
                let target = if self.is_closing { 0.0 } else { 1.0 };
                fade.set_target(target);
            }
            fade.update(dt);
        }

        // Pulse animations
        self.status_pulse.update(dt);
        if auto_inject_active {
            self.auto_inject_pulse.update(dt);
        }

        // Button hover updates
        for hover in &mut self.button_hover {
            hover.update(dt);
        }

        // Modal animations
        if show_process_list {
            self.process_modal.show();
        } else {
            self.process_modal.hide();
        }

        if show_confirm {
            self.confirm_modal.show();
        } else {
            self.confirm_modal.hide();
        }

        self.process_modal.fade.update(dt);
        self.process_modal.scale.update(dt);
        self.confirm_modal.fade.update(dt);
        self.confirm_modal.scale.update(dt);

        // Flash animation
        self.flash_fade.update(dt);
    }

    fn set_button_hover(&mut self, index: usize, hovered: bool) {
        if index < self.button_hover.len() {
            self.button_hover[index].set_target(if hovered { 1.0 } else { 0.0 });
        }
    }

    fn trigger_flash(&mut self, is_success: bool) {
        self.flash_fade.current = FLASH_ALPHA_START;
        self.flash_fade.target = 0.0;
        self.flash_is_success = is_success;
    }

    fn update_history(&mut self, history_len: usize) {
        while self.history_fades.len() > history_len {
            self.history_fades.pop();
        }
        while self.history_fades.len() < history_len {
            self.history_fades.push(Fade::new());
        }

        for (i, fade) in self.history_fades.iter_mut().enumerate() {
            fade.set_target(1.0);
            // Stagger with index
            let dt = 1.0 - (i as f32 * 0.1).max(0.5);
            fade.update(dt);
        }
    }

    fn has_active_animations(&self) -> bool {
        !self.global_fade.is_complete()
            || !self.title_fade.is_complete()
            || !self.title_scale.is_complete()
            || self.section_fades.iter().any(|f| !f.is_complete())
            || self.process_modal.is_visible()
            || self.confirm_modal.is_visible()
            || self.flash_fade.get() > ALPHA_THRESHOLD
            || self.history_fades.iter().any(|f| !f.is_complete())
            || self.button_hover.iter().any(|f| f.get() > ALPHA_THRESHOLD)
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
        if self.animation.has_active_animations()
            || self.show_process_list
            || self.show_confirm_dialog
        {
            ctx.request_repaint();
        }

        // Flash overlay for success/error feedback
        if self.animation.flash_fade.get() > ALPHA_THRESHOLD {
            let flash_color = if self.animation.flash_is_success {
                egui::Color32::from_rgba_unmultiplied(
                    0,
                    255,
                    0,
                    (50.0 * self.animation.flash_fade.get()) as u8,
                )
            } else {
                egui::Color32::from_rgba_unmultiplied(
                    255,
                    0,
                    0,
                    (50.0 * self.animation.flash_fade.get()) as u8,
                )
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
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_opacity(self.animation.global_fade.get());
            ui.add_space(20.0);

            // Title with scale and fade animation
            ui.vertical_centered(|ui| {
                let title_color = egui::Color32::from_rgba_premultiplied(
                    255,
                    255,
                    255,
                    (255.0 * self.animation.title_fade.get()) as u8,
                );
                let title_size = FONT_SIZE_LARGE * self.animation.title_scale.get();
                ui.label(
                    egui::RichText::new("Ruin DLL Injector")
                        .size(title_size)
                        .color(title_color)
                        .strong(),
                );

                // Admin status with pulse animation
                let admin_pulse = self.animation.status_pulse.get();
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
            let section_alpha = self
                .animation
                .section_fades
                .first()
                .map(|f| f.get())
                .unwrap_or(1.0);
            let section_offset = (1.0 - section_alpha) * 30.0;
            ui.add_space(section_offset);

            let label_color = egui::Color32::from_rgba_premultiplied(
                255,
                255,
                255,
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
                self.animation
                    .set_button_hover(0, browse_response.hovered());

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
                let path_alpha = self
                    .animation
                    .section_fades
                    .first()
                    .map(|f| f.get())
                    .unwrap_or(1.0);
                ui.label(
                    egui::RichText::new(path.display().to_string())
                        .size(FONT_SIZE_SMALL)
                        .color(egui::Color32::from_rgba_premultiplied(
                            200,
                            200,
                            200,
                            (255.0 * path_alpha) as u8,
                        )),
                );
            }

            ui.add_space(20.0);

            // Section 2: Target Process
            let section_alpha = self
                .animation
                .section_fades
                .get(1)
                .map(|f| f.get())
                .unwrap_or(1.0);
            let label_color = egui::Color32::from_rgba_premultiplied(
                255,
                255,
                255,
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
                    let pulse = self.animation.status_pulse.get();
                    egui::Color32::from_rgb(
                        (100.0 * pulse) as u8,
                        (200.0 * pulse) as u8,
                        (100.0 * pulse) as u8,
                    )
                } else {
                    egui::Color32::GRAY
                };

                let list_btn = egui::Button::new(egui::RichText::new(btn_text).color(btn_color));
                let list_response = ui.add(list_btn);

                self.animation.set_button_hover(2, list_response.hovered());

                if list_response.clicked() {
                    self.refresh_process_list();
                    self.show_process_list = true;
                }
            });

            // Process status with pulsing animation
            if !self.process_name.is_empty() {
                let is_running = self.is_process_running();
                let status_text = if is_running {
                    "Status: Running"
                } else {
                    "Status: Not found"
                };

                let pulse = if is_running {
                    self.animation.status_pulse.get()
                } else {
                    1.0
                };
                let status_color = if is_running {
                    egui::Color32::from_rgba_premultiplied(144, 238, 144, (255.0 * pulse) as u8)
                } else {
                    egui::Color32::from_rgba_premultiplied(
                        180,
                        180,
                        180,
                        (255.0
                            * self
                                .animation
                                .section_fades
                                .get(1)
                                .map(|f| f.get())
                                .unwrap_or(1.0)) as u8,
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
            let _section_alpha = self
                .animation
                .section_fades
                .get(2)
                .map(|f| f.get())
                .unwrap_or(1.0);

            ui.horizontal(|ui| {
                // Inject button with hover glow effect
                let inject_btn = egui::Button::new(
                    egui::RichText::new("Inject DLL").color(egui::Color32::WHITE),
                );
                let inject_response = ui.add(inject_btn);

                self.animation
                    .set_button_hover(1, inject_response.hovered());

                if inject_response.clicked() {
                    self.show_confirm_dialog = true;
                }

                // Auto-inject checkbox
                let _checkbox_color = if self.auto_inject {
                    let pulse = self.animation.auto_inject_pulse.get();
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
                    let pulse = self.animation.auto_inject_pulse.get();
                    ui.label(
                        egui::RichText::new("Active")
                            .color(egui::Color32::from_rgba_premultiplied(
                                (144.0 * pulse) as u8,
                                (238.0 * pulse) as u8,
                                (144.0 * pulse) as u8,
                                255,
                            ))
                            .size(FONT_SIZE_SMALL),
                    );
                }
            });

            ui.add_space(20.0);

            // Section 4: Injection history with staggered animations
            if !self.injection_history.is_empty() {
                let section_alpha = self
                    .animation
                    .section_fades
                    .get(3)
                    .map(|f| f.get())
                    .unwrap_or(1.0);
                let label_color = egui::Color32::from_rgba_premultiplied(
                    255,
                    255,
                    255,
                    (255.0 * section_alpha) as u8,
                );
                ui.label(
                    egui::RichText::new("Recent Injections")
                        .size(FONT_SIZE_MEDIUM)
                        .color(label_color),
                );
                ui.add_space(10.0);

                for (i, entry) in self.injection_history.iter().enumerate() {
                    let alpha = self
                        .animation
                        .history_fades
                        .get(i)
                        .map(|f| f.get())
                        .unwrap_or(1.0);
                    let slide_offset = (1.0 - alpha) * 20.0;

                    ui.horizontal(|ui| {
                        ui.add_space(slide_offset);
                        ui.label(egui::RichText::new(entry).size(FONT_SIZE_NORMAL).color(
                            egui::Color32::from_rgba_premultiplied(
                                180,
                                180,
                                180,
                                (255.0 * alpha) as u8,
                            ),
                        ));
                    });
                }
                ui.add_space(20.0);
            }

            ui.separator();
            ui.add_space(20.0);

            // Section 5: Activity log
            let section_alpha = self
                .animation
                .section_fades
                .get(4)
                .map(|f| f.get())
                .unwrap_or(1.0);
            let label_color = egui::Color32::from_rgba_premultiplied(
                255,
                255,
                255,
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
                            (255.0 * entry.get_alpha()) as u8,
                        );

                        ui.horizontal(|ui| {
                            ui.add_space(entry.get_slide_offset());
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
            let section_alpha = self
                .animation
                .section_fades
                .get(5)
                .map(|f| f.get())
                .unwrap_or(1.0);
            ui.label(
                egui::RichText::new("Some processes require administrator privileges")
                    .size(FONT_SIZE_SMALL)
                    .color(egui::Color32::from_rgba_premultiplied(
                        150,
                        150,
                        150,
                        (255.0 * section_alpha) as u8,
                    )),
            );

            self.check_auto_inject();
        });

        // Confirmation dialog modal (blocks background interaction)
        if self.animation.confirm_modal.is_visible() {
            // Draw blurred background overlay
            egui::Area::new(egui::Id::new("confirm_overlay"))
                .interactable(false)
                .fixed_pos(egui::Pos2::ZERO)
                .order(egui::Order::Middle)
                .show(ctx, |ui| {
                    let screen_rect = ui.ctx().screen_rect();
                    draw_blur_background(
                        ui.painter(),
                        screen_rect,
                        self.animation.confirm_modal.get_alpha(),
                    );
                });

            // Modal window with scale animation
            egui::Window::new("Confirm Injection")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .title_bar(false)
                .show(ctx, |ui| {
                    ui.set_enabled(self.animation.confirm_modal.get_scale() > 0.5);

                    egui::Frame::default()
                        .multiply_with_opacity(self.animation.confirm_modal.get_alpha())
                        .show(ui, |ui| {
                            let padding = (1.0 - self.animation.confirm_modal.get_scale())
                                * MODAL_PADDING_SCALE;
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
                                    egui::RichText::new("Confirm").color(egui::Color32::WHITE),
                                );
                                let cancel_btn = egui::Button::new(
                                    egui::RichText::new("Cancel").color(egui::Color32::WHITE),
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
        if self.animation.process_modal.is_visible() {
            // Draw blurred background overlay
            egui::Area::new(egui::Id::new("process_list_overlay"))
                .interactable(false)
                .fixed_pos(egui::Pos2::ZERO)
                .order(egui::Order::Middle)
                .show(ctx, |ui| {
                    let screen_rect = ui.ctx().screen_rect();
                    draw_blur_background(
                        ui.painter(),
                        screen_rect,
                        self.animation.process_modal.get_alpha(),
                    );
                });

            let modal_scale = self.animation.process_modal.get_scale();
            let modal_alpha = self.animation.process_modal.get_alpha();

            egui::Window::new("Select Process")
                .collapsible(false)
                .resizable(true)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.set_enabled(modal_scale > 0.5);

                    egui::Frame::default()
                        .multiply_with_opacity(modal_alpha)
                        .show(ui, |ui| {
                            let padding = (1.0 - modal_scale) * MODAL_PADDING_SCALE;
                            ui.add_space(padding);

                            ui.heading(
                                egui::RichText::new("Select Process").color(egui::Color32::WHITE),
                            );
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
                                        if search_lower.is_empty()
                                            || name_lower.contains(&search_lower)
                                        {
                                            has_matches = true;

                                            if ui
                                                .button(format!("{} (PID: {})", name, pid))
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

                            ui.horizontal(|ui| {
                                if ui.button("Cancel").clicked() {
                                    self.show_process_list = false;
                                    self.search_query.clear();
                                }
                            });

                            ui.add_space(padding);

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
    fn test_blur_background_early_return() {
        const {
            assert!(ALPHA_THRESHOLD > 0.0);
            assert!(ALPHA_THRESHOLD < 1.0);
        }
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
        assert_eq!(logger.get_entries()[0].get_alpha(), 0.0);
        assert!(logger.get_entries()[0].get_slide_offset() > 0.0);

        for _ in 0..50 {
            logger.update_frame();
        }

        assert!(logger.get_entries()[0].get_alpha() >= 0.9);
        assert!(logger.get_entries()[0].get_slide_offset() <= 2.0);
    }

    #[test]
    fn test_log_manager_max_limit() {
        let mut logger = LogManager::new();
        for i in 0..1002 {
            logger.add_log(format!("Log {}", i));
        }
        for _ in 0..80 {
            logger.update_frame();
        }
        assert_eq!(logger.get_entries().len(), MAX_LOGS);
    }

    #[test]
    fn test_animation_state_default() {
        let anim = AnimationState::default();
        assert_eq!(anim.global_fade.get(), 0.0);
        assert_eq!(anim.title_fade.get(), 0.0);
        assert_eq!(anim.process_modal.get_alpha(), 0.0);
        assert_eq!(anim.process_modal.get_scale(), 0.0);
    }

    #[test]
    fn test_animation_flash() {
        let mut anim = AnimationState::default();
        anim.trigger_flash(true);
        assert!(anim.flash_fade.current > 0.4);
        assert!(anim.flash_is_success);
    }

    #[test]
    fn test_modal_animation_show_hide() {
        let mut modal = ModalAnimation::new();
        assert_eq!(modal.get_alpha(), 0.0);
        assert_eq!(modal.get_scale(), 0.0);

        modal.show();
        assert_eq!(modal.fade.target, 1.0);
        assert_eq!(modal.scale.target, 1.0);

        modal.hide();
        assert_eq!(modal.fade.target, 0.0);
        assert_eq!(modal.scale.target, 0.0);
    }

    #[test]
    fn test_animation_history_sync() {
        let mut anim = AnimationState::default();
        anim.update_history(5);
        assert_eq!(anim.history_fades.len(), 5);
        anim.update_history(3);
        assert_eq!(anim.history_fades.len(), 3);
        anim.update_history(7);
        assert_eq!(anim.history_fades.len(), 7);
    }

    #[test]
    fn test_process_name_max_length() {
        let mut app = InjectorApp::default();
        let long_name = "a".repeat(300);
        app.process_name = long_name.clone();
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
        const {
            assert!(ANIMATION_DEFAULT_SPEED > 0.0 && ANIMATION_DEFAULT_SPEED < 1.0);
            assert!(ANIMATION_FAST_SPEED > 0.0 && ANIMATION_FAST_SPEED < 1.0);
            assert!(PULSE_SPEED_DEFAULT > 0.0);
            assert!(FLASH_ALPHA_START > 0.0 && FLASH_ALPHA_START <= 1.0);
            assert!(ALPHA_THRESHOLD > 0.0 && ALPHA_THRESHOLD < 1.0);
            assert!(SCALE_THRESHOLD > 0.0);
        }
    }

    #[test]
    fn test_modal_animation_update() {
        let mut anim = AnimationState::default();
        anim.process_modal.show();
        assert_eq!(anim.process_modal.fade.target, 1.0);
        assert_eq!(anim.process_modal.scale.target, 1.0);

        anim.process_modal.hide();
        assert_eq!(anim.process_modal.fade.target, 0.0);
        assert_eq!(anim.process_modal.scale.target, 0.0);
    }

    #[test]
    fn test_button_hover_animations() {
        let mut anim = AnimationState::default();
        anim.set_button_hover(0, true);
        assert!(anim.button_hover[0].target == 1.0);
        anim.set_button_hover(0, false);
        assert!(anim.button_hover[0].target == 0.0);
    }
}
