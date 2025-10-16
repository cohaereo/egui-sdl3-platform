use egui::{Modifiers, Pos2};
use sdl3::{
    event::{Event, WindowEvent},
    mouse::{Cursor, MouseButton, SystemCursor},
};

use crate::conversions::ToEguiKey;

/// The sdl3 platform for egui
pub struct Platform {
    // The cursors for the platform
    cursor: Option<Cursor>,
    system_cursor: SystemCursor,
    // The position of the mouse pointer
    pointer_pos: Pos2,
    // The egui modifiers
    modifiers: Modifiers,
    // The raw input
    raw_input: egui::RawInput,

    start_time: std::time::Instant,
    pixels_per_point: f32,

    // The egui context
    egui_ctx: egui::Context,
}

impl Platform {
    /// Construct a new [`Platform`]
    pub fn new(sdl: &sdl3::Sdl, window: &sdl3::video::Window) -> crate::Result<Self> {
        sdl.video()?.text_input().start(window);
        Ok(Self {
            cursor: Cursor::from_system(SystemCursor::Arrow)
                .map_err(|e| log::warn!("Failed to get cursor from systems cursor: {}", e))
                .ok(),
            system_cursor: SystemCursor::Arrow,
            pointer_pos: Pos2::ZERO,
            raw_input: egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::Vec2 {
                        x: window.size_in_pixels().0 as f32,
                        y: window.size_in_pixels().1 as f32,
                    },
                )),
                ..Default::default()
            },
            pixels_per_point: 1.0,
            modifiers: Modifiers::default(),
            egui_ctx: egui::Context::default(),
            start_time: std::time::Instant::now(),
        })
    }

    /// Handle a sdl3 event
    pub fn handle_event(&mut self, event: &Event, video: &sdl3::VideoSubsystem) {
        match event {
            // Handle reizing
            Event::Window {
                win_event: WindowEvent::Resized(w, h),
                ..
            } => {
                self.raw_input.screen_rect = Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::Vec2 {
                        x: *w as f32,
                        y: *h as f32,
                    },
                ));
            }
            // Handle the mouse button being held down
            Event::MouseButtonDown { mouse_btn, .. } => {
                let btn = match mouse_btn {
                    MouseButton::Left => Some(egui::PointerButton::Primary),
                    MouseButton::Middle => Some(egui::PointerButton::Middle),
                    MouseButton::Right => Some(egui::PointerButton::Secondary),
                    _ => None,
                };
                if let Some(btn) = btn {
                    self.raw_input.events.push(egui::Event::PointerButton {
                        pos: self.pointer_pos,
                        button: btn,
                        pressed: true,
                        modifiers: self.modifiers,
                    });
                }
                self.egui_ctx.wants_pointer_input();
            }
            // Handle the mouse button being released
            Event::MouseButtonUp { mouse_btn, .. } => {
                let btn = match mouse_btn {
                    MouseButton::Left => Some(egui::PointerButton::Primary),
                    MouseButton::Middle => Some(egui::PointerButton::Middle),
                    MouseButton::Right => Some(egui::PointerButton::Secondary),
                    _ => None,
                };
                if let Some(btn) = btn {
                    self.raw_input.events.push(egui::Event::PointerButton {
                        pos: self.pointer_pos,
                        button: btn,
                        pressed: false,
                        modifiers: self.modifiers,
                    });
                }
                self.egui_ctx.wants_pointer_input();
            }
            // Handle mouse motion
            Event::MouseMotion { x, y, .. } => {
                // Update the pointer position
                self.pointer_pos = egui::Pos2::new(*x, *y) / self.pixels_per_point;
                self.raw_input
                    .events
                    .push(egui::Event::PointerMoved(self.pointer_pos));
                self.egui_ctx.wants_pointer_input();
            }
            // Handle the mouse scrolling
            Event::MouseWheel { x, y, .. } => {
                // Calculate the delta
                let delta = egui::Vec2::new(*x, *y) * 32.0;

                self.raw_input.events.push(egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta,
                    modifiers: egui::Modifiers::NONE,
                });
                self.egui_ctx.wants_pointer_input();
            }

            // Handle a key being pressed
            Event::KeyDown {
                keycode, keymod, ..
            } => {
                // Make sure there is a keycode
                if let Some(keycode) = keycode {
                    // Update modifiers
                    use sdl3::keyboard::Mod;
                    let alt = keymod.contains(Mod::LALTMOD) || keymod.contains(Mod::RALTMOD);
                    let ctrl = keymod.contains(Mod::LCTRLMOD) || keymod.contains(Mod::RCTRLMOD);
                    let shift = keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD);
                    let mac_cmd = keymod.contains(Mod::LGUIMOD);
                    let command = keymod.contains(Mod::LCTRLMOD) || keymod.contains(Mod::LGUIMOD);

                    self.modifiers = Modifiers {
                        alt,
                        ctrl,
                        shift,
                        mac_cmd,
                        command,
                    };
                    self.raw_input.modifiers = self.modifiers;

                    // Convert the keycode to an egui key
                    if let Some(key) = keycode.to_egui_key() {
                        if self.modifiers.ctrl {
                            match key {
                                egui::Key::C => self.raw_input.events.push(egui::Event::Copy),
                                egui::Key::X => self.raw_input.events.push(egui::Event::Cut),
                                egui::Key::V => {
                                    let clipboard = video.clipboard();
                                    if clipboard.has_clipboard_text() {
                                        self.raw_input.events.push(egui::Event::Paste(
                                            clipboard.clipboard_text().unwrap(),
                                        ));
                                    }
                                }
                                _ => {}
                            }
                        }

                        // Push the event
                        self.raw_input.events.push(egui::Event::Key {
                            key,
                            physical_key: Some(key),
                            pressed: true,
                            repeat: false,
                            modifiers: self.modifiers,
                        });
                    }
                }
                self.egui_ctx.wants_keyboard_input();
            }
            // Handle a key being released
            Event::KeyUp {
                keycode, keymod, ..
            } => {
                // Make sure there is a keycode
                if let Some(keycode) = keycode {
                    // Update modifiers
                    use sdl3::keyboard::Mod;
                    let alt = keymod.contains(Mod::LALTMOD) || keymod.contains(Mod::RALTMOD);
                    let ctrl = keymod.contains(Mod::LCTRLMOD) || keymod.contains(Mod::RCTRLMOD);
                    let shift = keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD);
                    let mac_cmd = keymod.contains(Mod::LGUIMOD);
                    let command = keymod.contains(Mod::LCTRLMOD) || keymod.contains(Mod::LGUIMOD);

                    self.modifiers = Modifiers {
                        alt,
                        ctrl,
                        shift,
                        mac_cmd,
                        command,
                    };
                    self.raw_input.modifiers = self.modifiers;

                    // Convert the keycode to an egui key
                    if let Some(key) = keycode.to_egui_key() {
                        // Push the event
                        self.raw_input.events.push(egui::Event::Key {
                            key,
                            physical_key: Some(key),
                            pressed: false,
                            repeat: false,
                            modifiers: self.modifiers,
                        });
                    }
                }
                self.egui_ctx.wants_keyboard_input();
            }
            // Handle text input
            Event::TextInput { text, .. } => {
                self.raw_input.events.push(egui::Event::Text(text.clone()));
                self.egui_ctx.wants_keyboard_input();
            }

            _ => {}
        }
    }

    pub fn context(&self) -> &egui::Context {
        &self.egui_ctx
    }

    /// Return the processed context
    pub fn begin_frame(&mut self, window: &sdl3::video::Window) -> egui::Context {
        let screen_size = window.size();
        self.pixels_per_point = window.display_scale();
        // Set the pixels per point
        self.egui_ctx.set_pixels_per_point(self.pixels_per_point);
        self.raw_input.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(screen_size.0 as f32, screen_size.1 as f32) / self.pixels_per_point,
        ));
        self.raw_input.time = Some(self.start_time.elapsed().as_secs_f64());

        // Begin the frame
        self.egui_ctx.begin_pass(self.raw_input.take());
        // Return the ctx
        self.egui_ctx.clone()
    }

    /// Stop drawing the egui frame and return the full output
    pub fn end_frame(
        &mut self,
        video: &mut sdl3::VideoSubsystem,
    ) -> crate::Result<egui::FullOutput> {
        // Get the egui output
        let output = self.egui_ctx.end_pass();

        for c in &output.platform_output.commands {
            if let egui::OutputCommand::CopyText(text) = c {
                if let Err(e) = video.clipboard().set_clipboard_text(text) {
                    log::error!("Failed to assign text to clipboard: {}", e);
                }
            }
        }

        if let Some(cursor) = &mut self.cursor {
            // Update the cursor icon
            let new_cursor = match output.platform_output.cursor_icon {
                egui::CursorIcon::Crosshair => SystemCursor::Crosshair,
                egui::CursorIcon::Default => SystemCursor::Arrow,
                egui::CursorIcon::Grab => SystemCursor::Hand,
                egui::CursorIcon::Grabbing => SystemCursor::SizeAll,
                egui::CursorIcon::Move => SystemCursor::SizeAll,
                egui::CursorIcon::PointingHand => SystemCursor::Hand,
                egui::CursorIcon::ResizeHorizontal => SystemCursor::SizeWE,
                egui::CursorIcon::ResizeNeSw => SystemCursor::SizeNESW,
                egui::CursorIcon::ResizeNwSe => SystemCursor::SizeNWSE,
                egui::CursorIcon::ResizeVertical => SystemCursor::SizeNS,
                egui::CursorIcon::Text => SystemCursor::IBeam,
                egui::CursorIcon::NotAllowed | egui::CursorIcon::NoDrop => SystemCursor::No,
                egui::CursorIcon::Wait => SystemCursor::Wait,
                _ => SystemCursor::Arrow,
            };

            if self.system_cursor != new_cursor {
                self.system_cursor = new_cursor;
                *cursor = Cursor::from_system(new_cursor)?;
                cursor.set();
            }
        }

        Ok(output)
    }

    /// Tessellate the egui frame
    pub fn tessellate(&self, full_output: &egui::FullOutput) -> Vec<egui::ClippedPrimitive> {
        self.egui_ctx
            .tessellate(full_output.shapes.clone(), self.egui_ctx.pixels_per_point())
    }
}
