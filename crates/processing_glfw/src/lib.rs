use bevy::input::keyboard::{KeyCode, NativeKeyCode};
use bevy::input::mouse::MouseButton;
use bevy::math::{IRect, IVec2};
use bevy::prelude::Entity;
use bevy::window::{
    Monitor as BevyMonitor, MonitorSelection, PrimaryMonitor, VideoMode as BevyVideoMode,
    Window as BevyWindow, WindowLevel as BevyWindowLevel, WindowMode as BevyWindowMode,
    WindowPosition,
};
use glfw::{Action, Glfw, GlfwReceiver, PWindow, WindowEvent, WindowMode};
use processing_core::app_mut;
use processing_core::error::Result;
use processing_input::{
    input_cursor_grab_mode, input_cursor_visible, input_flush, input_set_char,
    input_set_cursor_enter, input_set_cursor_leave, input_set_focus, input_set_key,
    input_set_mouse_button, input_set_mouse_move, input_set_scroll,
};
use processing_render::surface::{MonitorWorkarea, WindowControls};

pub struct GlfwContext {
    glfw: Glfw,
    window: PWindow,
    events: GlfwReceiver<(f64, WindowEvent)>,
    surface: Option<Entity>,
    last_applied: AppliedWindow,
    windowed_geometry: Option<(i32, i32, u32, u32)>,
}

/// What we last pushed to the OS window, diffed against [`BevyWindow`] each tick so we
/// only call into GLFW when something actually changed.
#[derive(Clone, Debug)]
struct AppliedWindow {
    title: String,
    position: IVec2,
    size: bevy::math::UVec2,
    visible: bool,
    resizable: bool,
    decorations: bool,
    window_level: BevyWindowLevel,
    fullscreen_on: Option<Entity>,
    opacity: f32,
}

impl Default for AppliedWindow {
    fn default() -> Self {
        Self {
            title: String::new(),
            position: IVec2::ZERO,
            size: bevy::math::UVec2::ZERO,
            visible: true,
            resizable: true,
            decorations: true,
            window_level: BevyWindowLevel::Normal,
            fullscreen_on: None,
            opacity: 1.0,
        }
    }
}

impl GlfwContext {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

        glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        glfw.window_hint(glfw::WindowHint::Visible(false));
        glfw.window_hint(glfw::WindowHint::TransparentFramebuffer(true));

        let (mut window, events) = glfw
            .create_window(width, height, "Processing", WindowMode::Windowed)
            .unwrap();

        window.set_all_polling(true);
        window.show();

        Ok(Self {
            glfw,
            window,
            events,
            surface: None,
            last_applied: AppliedWindow::default(),
            windowed_geometry: None,
        })
    }

    fn sync_monitors(&mut self) {
        let primary_name = self
            .glfw
            .with_primary_monitor(|_, monitor| monitor.and_then(|m| m.get_name()));

        self.glfw.with_connected_monitors(|_, monitors| {
            let _ = app_mut(|app| {
                let world = app.world_mut();
                let mut existing: std::collections::HashMap<String, Entity> = world
                    .iter_entities()
                    .filter_map(|e| {
                        let name = e.get::<BevyMonitor>()?.name.clone()?;
                        Some((name, e.id()))
                    })
                    .collect();

                for monitor in monitors {
                    let name = monitor.get_name();
                    let video_mode = monitor.get_video_mode();
                    let (width, height) = video_mode
                        .as_ref()
                        .map(|v| (v.width, v.height))
                        .unwrap_or((0, 0));
                    let refresh_millihz = video_mode.as_ref().map(|v| v.refresh_rate * 1000);
                    let (x, y) = monitor.get_pos();
                    let (wx, wy, ww, wh) = monitor.get_workarea();
                    let (scale, _) = monitor.get_content_scale();
                    let position = IVec2::new(x, y);
                    let workarea =
                        IRect::from_corners(IVec2::new(wx, wy), IVec2::new(wx + ww, wy + wh));
                    let video_modes: Vec<BevyVideoMode> = monitor
                        .get_video_modes()
                        .into_iter()
                        .map(|v| BevyVideoMode {
                            physical_size: bevy::math::UVec2::new(v.width, v.height),
                            bit_depth: (v.red_bits + v.green_bits + v.blue_bits) as u16,
                            refresh_rate_millihertz: v.refresh_rate * 1000,
                        })
                        .collect();

                    let entity = match name.as_ref().and_then(|n| existing.remove(n)) {
                        Some(entity) => {
                            if let Some(mut bevy_monitor) = world.get_mut::<BevyMonitor>(entity) {
                                bevy_monitor.physical_width = width;
                                bevy_monitor.physical_height = height;
                                bevy_monitor.physical_position = position;
                                bevy_monitor.refresh_rate_millihertz = refresh_millihz;
                                bevy_monitor.scale_factor = scale as f64;
                                bevy_monitor.video_modes = video_modes;
                            }
                            match world.get_mut::<MonitorWorkarea>(entity) {
                                Some(mut current) => current.0 = workarea,
                                None => {
                                    world.entity_mut(entity).insert(MonitorWorkarea(workarea));
                                }
                            }
                            entity
                        }
                        None => world
                            .spawn((
                                BevyMonitor {
                                    name: name.clone(),
                                    physical_height: height,
                                    physical_width: width,
                                    physical_position: position,
                                    refresh_rate_millihertz: refresh_millihz,
                                    scale_factor: scale as f64,
                                    video_modes,
                                },
                                MonitorWorkarea(workarea),
                            ))
                            .id(),
                    };

                    let is_primary = name.is_some() && name == primary_name;
                    let was_primary = world.get::<PrimaryMonitor>(entity).is_some();
                    match (is_primary, was_primary) {
                        (true, false) => {
                            world.entity_mut(entity).insert(PrimaryMonitor);
                        }
                        (false, true) => {
                            world.entity_mut(entity).remove::<PrimaryMonitor>();
                        }
                        _ => {}
                    }
                }

                for (_, entity) in existing {
                    world.entity_mut(entity).despawn();
                }

                Ok(())
            });
        });
    }

    #[cfg(target_os = "macos")]
    pub fn create_surface(&mut self, width: u32, height: u32) -> Result<Entity> {
        use processing_render::surface_create_macos;
        let (scale_factor, _) = self.window.get_content_scale();
        let entity = surface_create_macos(
            self.window.get_cocoa_window() as u64,
            width,
            height,
            scale_factor,
        )?;
        self.surface = Some(entity);
        Ok(entity)
    }

    #[cfg(target_os = "windows")]
    pub fn create_surface(&mut self, width: u32, height: u32) -> Result<Entity> {
        use processing_render::surface_create_windows;
        let (scale_factor, _) = self.window.get_content_scale();
        let entity = surface_create_windows(
            self.window.get_win32_window() as u64,
            width,
            height,
            scale_factor,
        )?;
        self.surface = Some(entity);
        Ok(entity)
    }

    #[cfg(all(target_os = "linux", feature = "wayland"))]
    pub fn create_surface(&mut self, width: u32, height: u32) -> Result<Entity> {
        use processing_render::surface_create_wayland;
        let (scale_factor, _) = self.window.get_content_scale();
        let entity = surface_create_wayland(
            self.window.get_wayland_window() as u64,
            self.glfw.get_wayland_display() as u64,
            width,
            height,
            scale_factor,
        )?;
        self.surface = Some(entity);
        Ok(entity)
    }

    #[cfg(all(target_os = "linux", feature = "x11"))]
    pub fn create_surface(&mut self, width: u32, height: u32) -> Result<Entity> {
        use processing_render::surface_create_x11;
        let (scale_factor, _) = self.window.get_content_scale();
        let entity = surface_create_x11(
            self.window.get_x11_window() as u64,
            self.glfw.get_x11_display() as u64,
            width,
            height,
            scale_factor,
        )?;
        self.surface = Some(entity);
        Ok(entity)
    }

    pub fn poll_events(&mut self) -> bool {
        self.glfw.poll_events();

        let surface = match self.surface {
            Some(s) => s,
            None => {
                for (_, event) in glfw::flush_messages(&self.events) {
                    if event == WindowEvent::Close {
                        self.window.hide();
                        return false;
                    }
                }
                if self.window.should_close() {
                    self.window.hide();
                    return false;
                }
                return true;
            }
        };

        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                WindowEvent::Close => {
                    self.window.hide();
                    return false;
                }
                WindowEvent::CursorPos(x, y) => {
                    input_set_mouse_move(surface, x as f32, y as f32).unwrap();
                }
                WindowEvent::MouseButton(button, action, _mods) => {
                    if let Some(btn) = glfw_button_to_bevy(button) {
                        input_set_mouse_button(surface, btn, action == Action::Press).unwrap();
                    }
                }
                WindowEvent::Scroll(x, y) => {
                    input_set_scroll(surface, x as f32, y as f32).unwrap();
                }
                WindowEvent::Key(key, _scancode, action, _mods) => {
                    if let Some(kc) = glfw_key_to_bevy(key) {
                        input_set_key(
                            surface,
                            kc,
                            action == Action::Press || action == Action::Repeat,
                        )
                        .unwrap();
                    }
                }
                WindowEvent::Char(ch) => {
                    input_set_char(
                        surface,
                        KeyCode::Unidentified(NativeKeyCode::Unidentified),
                        ch,
                    )
                    .unwrap();
                }
                WindowEvent::CursorEnter(true) => {
                    input_set_cursor_enter(surface).unwrap();
                }
                WindowEvent::CursorEnter(false) => {
                    input_set_cursor_leave(surface).unwrap();
                }
                WindowEvent::Focus(focused) => {
                    input_set_focus(surface, focused).unwrap();
                }
                _ => {}
            }
        }

        if self.window.should_close() {
            self.window.hide();
            return false;
        }

        let Ok(_) = input_flush() else {
            return false;
        };
        self.sync_cursor(surface);
        self.sync_monitors();
        self.sync_window(surface);

        true
    }

    fn sync_window(&mut self, surface: Entity) {
        let Some(desired) = read_desired_window(surface) else {
            return;
        };

        self.apply_window(&desired);

        if desired.iconify {
            self.window.iconify();
        }
        if desired.restore {
            self.window.restore();
        }
        if desired.maximize {
            self.window.maximize();
        }
        if desired.focus {
            self.window.focus();
        }

        let (cx, cy) = self.window.get_pos();
        let (inset_l, inset_t, _, _) = self.window.get_frame_size();
        let frame_pos = IVec2::new(cx - inset_l, cy - inset_t);
        let _ = app_mut(|app| {
            let world = app.world_mut();
            if let Some(mut window) = world.get_mut::<BevyWindow>(surface) {
                window.position = WindowPosition::At(frame_pos);
            }
            if let Some(mut controls) = world.get_mut::<WindowControls>(surface) {
                controls.pending_iconify = false;
                controls.pending_restore = false;
                controls.pending_maximize = false;
                controls.pending_focus = false;
            }
            Ok(())
        });
        self.last_applied.position = frame_pos;
    }

    fn apply_window(&mut self, desired: &DesiredWindow) {
        let last = &mut self.last_applied;

        if desired.title != last.title {
            self.window.set_title(&desired.title);
            last.title.clone_from(&desired.title);
        }
        if let Some(pos) = desired.position
            && pos != last.position
        {
            let (inset_l, inset_t, _, _) = self.window.get_frame_size();
            self.window.set_pos(pos.x + inset_l, pos.y + inset_t);
            last.position = pos;
        }
        if desired.size != last.size && desired.size.x > 0 && desired.size.y > 0 {
            self.window
                .set_size(desired.size.x as i32, desired.size.y as i32);
            last.size = desired.size;
        }
        if desired.visible != last.visible {
            if desired.visible {
                self.window.show();
            } else {
                self.window.hide();
            }
            last.visible = desired.visible;
        }
        if desired.resizable != last.resizable {
            self.window.set_resizable(desired.resizable);
            last.resizable = desired.resizable;
        }
        if desired.decorations != last.decorations {
            self.window.set_decorated(desired.decorations);
            last.decorations = desired.decorations;
        }
        if desired.window_level != last.window_level {
            self.window
                .set_floating(matches!(desired.window_level, BevyWindowLevel::AlwaysOnTop));
            last.window_level = desired.window_level;
        }
        if let Some(opacity) = desired.opacity
            && (opacity - last.opacity).abs() > f32::EPSILON
        {
            self.window.set_opacity(opacity);
            last.opacity = opacity;
        }
        if desired.fullscreen_on != last.fullscreen_on {
            self.apply_fullscreen(desired.fullscreen_on);
        }
    }

    fn apply_fullscreen(&mut self, target: Option<Entity>) {
        match target {
            Some(monitor_entity) => {
                if self.last_applied.fullscreen_on.is_none() {
                    let (x, y) = self.window.get_pos();
                    let (w, h) = self.window.get_size();
                    self.windowed_geometry = Some((x, y, w as u32, h as u32));
                }
                let target_name = monitor_name(monitor_entity);
                let window = &mut self.window;
                let applied = self.glfw.with_connected_monitors(|_, monitors| {
                    let Some(monitor) = monitors
                        .iter()
                        .find(|m| m.get_name() == target_name)
                        .map(|m| &**m)
                    else {
                        return false;
                    };
                    let (w, h, refresh) = monitor
                        .get_video_mode()
                        .map(|v| (v.width, v.height, Some(v.refresh_rate)))
                        .unwrap_or((1920, 1080, None));
                    window.set_monitor(WindowMode::FullScreen(monitor), 0, 0, w, h, refresh);
                    true
                });
                self.last_applied.fullscreen_on = applied.then_some(monitor_entity);
            }
            None => {
                let (x, y, w, h) = self.windowed_geometry.take().unwrap_or_else(|| {
                    let (x, y) = self.window.get_pos();
                    let (w, h) = self.window.get_size();
                    (x, y, w as u32, h as u32)
                });
                self.window
                    .set_monitor(WindowMode::Windowed, x, y, w, h, None);
                self.last_applied.fullscreen_on = None;
            }
        }
    }

    pub fn content_scale(&self) -> f32 {
        let (s, _) = self.window.get_content_scale();
        s
    }

    fn sync_cursor(&mut self, surface: Entity) {
        use bevy::window::CursorGrabMode;

        let grab = input_cursor_grab_mode(surface).unwrap_or(CursorGrabMode::None);
        let visible = input_cursor_visible(surface).unwrap_or(true);

        let mode = match grab {
            CursorGrabMode::Locked | CursorGrabMode::Confined => glfw::CursorMode::Disabled,
            CursorGrabMode::None if !visible => glfw::CursorMode::Hidden,
            CursorGrabMode::None => glfw::CursorMode::Normal,
        };

        if self.window.get_cursor_mode() != mode {
            self.window.set_cursor_mode(mode);
        }
    }
}

#[derive(Clone, Debug)]
struct DesiredWindow {
    title: String,
    position: Option<IVec2>,
    size: bevy::math::UVec2,
    visible: bool,
    resizable: bool,
    decorations: bool,
    window_level: BevyWindowLevel,
    fullscreen_on: Option<Entity>,
    opacity: Option<f32>,
    iconify: bool,
    restore: bool,
    maximize: bool,
    focus: bool,
}

fn read_desired_window(surface: Entity) -> Option<DesiredWindow> {
    app_mut(|app| {
        let world = app.world();
        let Some(window) = world.get::<BevyWindow>(surface) else {
            return Ok(None);
        };
        let controls = world
            .get::<WindowControls>(surface)
            .cloned()
            .unwrap_or_default();
        let fullscreen_on = match window.mode {
            BevyWindowMode::Windowed => None,
            BevyWindowMode::BorderlessFullscreen(sel) | BevyWindowMode::Fullscreen(sel, _) => {
                resolve_monitor(world, sel)
            }
        };
        Ok(Some(DesiredWindow {
            title: window.title.clone(),
            position: match window.position {
                WindowPosition::At(p) => Some(p),
                _ => None,
            },
            size: bevy::math::UVec2::new(
                window.resolution.physical_width(),
                window.resolution.physical_height(),
            ),
            visible: window.visible,
            resizable: window.resizable,
            decorations: window.decorations,
            window_level: window.window_level,
            fullscreen_on,
            opacity: controls.opacity,
            iconify: controls.pending_iconify,
            restore: controls.pending_restore,
            maximize: controls.pending_maximize,
            focus: controls.pending_focus,
        }))
    })
    .ok()
    .flatten()
}

fn resolve_monitor(world: &bevy::ecs::world::World, sel: MonitorSelection) -> Option<Entity> {
    match sel {
        MonitorSelection::Entity(e) => world.get::<BevyMonitor>(e).map(|_| e),
        MonitorSelection::Primary | MonitorSelection::Current => world
            .iter_entities()
            .find(|e| e.contains::<PrimaryMonitor>() && e.contains::<BevyMonitor>())
            .map(|e| e.id()),
        MonitorSelection::Index(idx) => {
            let mut entities: Vec<Entity> = world
                .iter_entities()
                .filter(|e| e.contains::<BevyMonitor>())
                .map(|e| e.id())
                .collect();
            entities.sort();
            entities.get(idx).copied()
        }
    }
}

fn monitor_name(entity: Entity) -> Option<String> {
    app_mut(|app| {
        Ok(app
            .world()
            .get::<BevyMonitor>(entity)
            .and_then(|m| m.name.clone()))
    })
    .ok()
    .flatten()
}

fn glfw_button_to_bevy(button: glfw::MouseButton) -> Option<MouseButton> {
    match button {
        glfw::MouseButtonLeft => Some(MouseButton::Left),
        glfw::MouseButtonRight => Some(MouseButton::Right),
        glfw::MouseButtonMiddle => Some(MouseButton::Middle),
        _ => None,
    }
}

fn glfw_key_to_bevy(key: glfw::Key) -> Option<KeyCode> {
    match key {
        glfw::Key::Space => Some(KeyCode::Space),
        glfw::Key::Apostrophe => Some(KeyCode::Quote),
        glfw::Key::Comma => Some(KeyCode::Comma),
        glfw::Key::Minus => Some(KeyCode::Minus),
        glfw::Key::Period => Some(KeyCode::Period),
        glfw::Key::Slash => Some(KeyCode::Slash),
        glfw::Key::Num0 => Some(KeyCode::Digit0),
        glfw::Key::Num1 => Some(KeyCode::Digit1),
        glfw::Key::Num2 => Some(KeyCode::Digit2),
        glfw::Key::Num3 => Some(KeyCode::Digit3),
        glfw::Key::Num4 => Some(KeyCode::Digit4),
        glfw::Key::Num5 => Some(KeyCode::Digit5),
        glfw::Key::Num6 => Some(KeyCode::Digit6),
        glfw::Key::Num7 => Some(KeyCode::Digit7),
        glfw::Key::Num8 => Some(KeyCode::Digit8),
        glfw::Key::Num9 => Some(KeyCode::Digit9),
        glfw::Key::Semicolon => Some(KeyCode::Semicolon),
        glfw::Key::Equal => Some(KeyCode::Equal),
        glfw::Key::A => Some(KeyCode::KeyA),
        glfw::Key::B => Some(KeyCode::KeyB),
        glfw::Key::C => Some(KeyCode::KeyC),
        glfw::Key::D => Some(KeyCode::KeyD),
        glfw::Key::E => Some(KeyCode::KeyE),
        glfw::Key::F => Some(KeyCode::KeyF),
        glfw::Key::G => Some(KeyCode::KeyG),
        glfw::Key::H => Some(KeyCode::KeyH),
        glfw::Key::I => Some(KeyCode::KeyI),
        glfw::Key::J => Some(KeyCode::KeyJ),
        glfw::Key::K => Some(KeyCode::KeyK),
        glfw::Key::L => Some(KeyCode::KeyL),
        glfw::Key::M => Some(KeyCode::KeyM),
        glfw::Key::N => Some(KeyCode::KeyN),
        glfw::Key::O => Some(KeyCode::KeyO),
        glfw::Key::P => Some(KeyCode::KeyP),
        glfw::Key::Q => Some(KeyCode::KeyQ),
        glfw::Key::R => Some(KeyCode::KeyR),
        glfw::Key::S => Some(KeyCode::KeyS),
        glfw::Key::T => Some(KeyCode::KeyT),
        glfw::Key::U => Some(KeyCode::KeyU),
        glfw::Key::V => Some(KeyCode::KeyV),
        glfw::Key::W => Some(KeyCode::KeyW),
        glfw::Key::X => Some(KeyCode::KeyX),
        glfw::Key::Y => Some(KeyCode::KeyY),
        glfw::Key::Z => Some(KeyCode::KeyZ),
        glfw::Key::LeftBracket => Some(KeyCode::BracketLeft),
        glfw::Key::Backslash => Some(KeyCode::Backslash),
        glfw::Key::RightBracket => Some(KeyCode::BracketRight),
        glfw::Key::GraveAccent => Some(KeyCode::Backquote),
        glfw::Key::Escape => Some(KeyCode::Escape),
        glfw::Key::Enter => Some(KeyCode::Enter),
        glfw::Key::Tab => Some(KeyCode::Tab),
        glfw::Key::Backspace => Some(KeyCode::Backspace),
        glfw::Key::Insert => Some(KeyCode::Insert),
        glfw::Key::Delete => Some(KeyCode::Delete),
        glfw::Key::Right => Some(KeyCode::ArrowRight),
        glfw::Key::Left => Some(KeyCode::ArrowLeft),
        glfw::Key::Down => Some(KeyCode::ArrowDown),
        glfw::Key::Up => Some(KeyCode::ArrowUp),
        glfw::Key::PageUp => Some(KeyCode::PageUp),
        glfw::Key::PageDown => Some(KeyCode::PageDown),
        glfw::Key::Home => Some(KeyCode::Home),
        glfw::Key::End => Some(KeyCode::End),
        glfw::Key::CapsLock => Some(KeyCode::CapsLock),
        glfw::Key::ScrollLock => Some(KeyCode::ScrollLock),
        glfw::Key::NumLock => Some(KeyCode::NumLock),
        glfw::Key::PrintScreen => Some(KeyCode::PrintScreen),
        glfw::Key::Pause => Some(KeyCode::Pause),
        glfw::Key::F1 => Some(KeyCode::F1),
        glfw::Key::F2 => Some(KeyCode::F2),
        glfw::Key::F3 => Some(KeyCode::F3),
        glfw::Key::F4 => Some(KeyCode::F4),
        glfw::Key::F5 => Some(KeyCode::F5),
        glfw::Key::F6 => Some(KeyCode::F6),
        glfw::Key::F7 => Some(KeyCode::F7),
        glfw::Key::F8 => Some(KeyCode::F8),
        glfw::Key::F9 => Some(KeyCode::F9),
        glfw::Key::F10 => Some(KeyCode::F10),
        glfw::Key::F11 => Some(KeyCode::F11),
        glfw::Key::F12 => Some(KeyCode::F12),
        glfw::Key::F13 => Some(KeyCode::F13),
        glfw::Key::F14 => Some(KeyCode::F14),
        glfw::Key::F15 => Some(KeyCode::F15),
        glfw::Key::F16 => Some(KeyCode::F16),
        glfw::Key::F17 => Some(KeyCode::F17),
        glfw::Key::F18 => Some(KeyCode::F18),
        glfw::Key::F19 => Some(KeyCode::F19),
        glfw::Key::F20 => Some(KeyCode::F20),
        glfw::Key::F21 => Some(KeyCode::F21),
        glfw::Key::F22 => Some(KeyCode::F22),
        glfw::Key::F23 => Some(KeyCode::F23),
        glfw::Key::F24 => Some(KeyCode::F24),
        glfw::Key::F25 => Some(KeyCode::F25),
        glfw::Key::Kp0 => Some(KeyCode::Numpad0),
        glfw::Key::Kp1 => Some(KeyCode::Numpad1),
        glfw::Key::Kp2 => Some(KeyCode::Numpad2),
        glfw::Key::Kp3 => Some(KeyCode::Numpad3),
        glfw::Key::Kp4 => Some(KeyCode::Numpad4),
        glfw::Key::Kp5 => Some(KeyCode::Numpad5),
        glfw::Key::Kp6 => Some(KeyCode::Numpad6),
        glfw::Key::Kp7 => Some(KeyCode::Numpad7),
        glfw::Key::Kp8 => Some(KeyCode::Numpad8),
        glfw::Key::Kp9 => Some(KeyCode::Numpad9),
        glfw::Key::KpDecimal => Some(KeyCode::NumpadDecimal),
        glfw::Key::KpDivide => Some(KeyCode::NumpadDivide),
        glfw::Key::KpMultiply => Some(KeyCode::NumpadMultiply),
        glfw::Key::KpSubtract => Some(KeyCode::NumpadSubtract),
        glfw::Key::KpAdd => Some(KeyCode::NumpadAdd),
        glfw::Key::KpEnter => Some(KeyCode::NumpadEnter),
        glfw::Key::KpEqual => Some(KeyCode::NumpadEqual),
        glfw::Key::LeftShift => Some(KeyCode::ShiftLeft),
        glfw::Key::LeftControl => Some(KeyCode::ControlLeft),
        glfw::Key::LeftAlt => Some(KeyCode::AltLeft),
        glfw::Key::LeftSuper => Some(KeyCode::SuperLeft),
        glfw::Key::RightShift => Some(KeyCode::ShiftRight),
        glfw::Key::RightControl => Some(KeyCode::ControlRight),
        glfw::Key::RightAlt => Some(KeyCode::AltRight),
        glfw::Key::RightSuper => Some(KeyCode::SuperRight),
        glfw::Key::Menu => Some(KeyCode::ContextMenu),
        _ => None,
    }
}
