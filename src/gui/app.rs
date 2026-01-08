// app module specifying gui interface
use super::{editor::EditorPanel, state::EditorState, themes};
use egui::{Context, ViewportCommand};

pub struct QuickNotepadApp {
    state: EditorState,
    show_shortcuts: bool,
    show_save_dialog: bool,
    save_filename: String,
    // Flag to prevent text input when dialogs are open
    dialog_has_focus: bool,
}

impl QuickNotepadApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, file_path: Option<String>) -> Self {
        Self {
            state: EditorState::new(file_path),
            show_shortcuts: false,
            show_save_dialog: false,
            save_filename: String::new(),
            dialog_has_focus: false,
        }
    }

    fn menu_bar(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            // Updated to the modern non-deprecated builder pattern
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("📄 New (Ctrl+N)").clicked() {
                        self.state.tab_manager.new_tab();
                        ui.close();
                    }

                    if ui.button("💾 Save (Ctrl+S)").clicked() {
                        if self.state.current_filename().is_some() {
                            let _ = self.state.save();
                        } else {
                            self.show_save_dialog = true;
                        }
                        ui.close();
                    }

                    if ui.button("💾 Save As...").clicked() {
                        self.show_save_dialog = true;
                        ui.close();
                    }

                    ui.separator();

                    if ui.button("❌ Quit (Ctrl+Q)").clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                        ui.close();
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("↶ Undo (Ctrl+Z)").clicked() {
                        if let Some(op) = self.state.current_edit_history().undo() {
                            op.edit.reverse(&mut self.state.current_buffer_mut().lines);
                        }
                        ui.close();
                    }

                    if ui.button("↷ Redo (Ctrl+Y)").clicked() {
                        if let Some(op) = self.state.current_edit_history().redo() {
                            op.edit.apply(&mut self.state.current_buffer_mut().lines);
                        }
                        ui.close();
                    }

                    ui.separator();

                    if ui.button("📋 Copy (Ctrl+C)").clicked() {
                        self.state.copy_selection();
                        ui.close();
                    }

                    if ui.button("✂ Cut (Ctrl+X)").clicked() {
                        self.state.cut_selection();
                        ui.close();
                    }

                    if ui.button("📄 Paste (Ctrl+V)").clicked() {
                        self.state.paste_from_clipboard();
                        ui.close();
                    }

                    ui.separator();

                    if ui.button("🔍 Find (Ctrl+F)").clicked() {
                        self.state.search_active = true;
                        ui.close();
                    }

                    if ui.button("🔤 Select All (Ctrl+A)").clicked() {
                        self.state.select_all();
                        ui.close();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("⌨ Shortcuts").clicked() {
                        self.show_shortcuts = !self.show_shortcuts;
                        ui.close();
                    }
                });

                ui.menu_button("Tabs", |ui| {
                    for i in 1..=9 {
                        let tab_text = format!("Tab {} (Ctrl+{})", i, i);
                        if ui.button(tab_text).clicked() {
                            let _ = self.state.tab_manager.switch_to_tab(i);
                            ui.close();
                        }
                    }
                });
            });
        });
    }

    fn status_bar(&mut self, ctx: &Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Left side - filename
                let filename = self.state.current_filename().unwrap_or("[No Name]");
                let dirty = if self.state.has_unsaved_changes() {
                    "*"
                } else {
                    ""
                };
                ui.label(format!("{}{}", filename, dirty));

                ui.separator();

                // Position
                ui.label(format!(
                    "Ln {}, Col {}",
                    self.state.cursor_pos.line + 1,
                    self.state.cursor_pos.column + 1
                ));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("© Filip Domanski");
                    ui.separator();

                    // Line count
                    let line_count = self
                        .state
                        .current_buffer()
                        .lines
                        .iter()
                        .rposition(|l| !l.is_empty())
                        .map(|i| i + 1)
                        .unwrap_or(1);
                    ui.label(format!("Lines: {}", line_count));
                });
            });
        });
    }

    fn handle_shortcuts(&mut self, ctx: &Context) {
        if self.dialog_has_focus {
            return;
        }

        // File operations
        if ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl) {
            if self.state.current_filename().is_some() {
                let _ = self.state.save();
            } else {
                self.show_save_dialog = true;
                self.dialog_has_focus = true;
            }
        }

        if ctx.input(|i| i.key_pressed(egui::Key::N) && i.modifiers.ctrl) {
            self.state.tab_manager.new_tab();
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Q) && i.modifiers.ctrl) {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }

        // Edit operations
        if ctx.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.ctrl) {
            if let Some(op) = self.state.current_edit_history().undo() {
                op.edit.reverse(&mut self.state.current_buffer_mut().lines);
            }
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Y) && i.modifiers.ctrl) {
            if let Some(op) = self.state.current_edit_history().redo() {
                op.edit.apply(&mut self.state.current_buffer_mut().lines);
            }
        }

        if ctx.input(|i| i.key_pressed(egui::Key::F) && i.modifiers.ctrl) {
            self.state.search_active = true;
            self.dialog_has_focus = true;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::A) && i.modifiers.ctrl) {
            self.state.select_all();
        }

        if ctx.input(|i| i.key_pressed(egui::Key::C) && i.modifiers.ctrl) {
            self.state.copy_selection();
        }

        if ctx.input(|i| i.key_pressed(egui::Key::X) && i.modifiers.ctrl) {
            self.state.cut_selection();
        }

        if ctx.input(|i| i.key_pressed(egui::Key::V) && i.modifiers.ctrl) {
            self.state.paste_from_clipboard();
        }

        for i in 1..=9 {
            if ctx.input(|input| {
                input.key_pressed(match i {
                    1 => egui::Key::Num1, 2 => egui::Key::Num2, 3 => egui::Key::Num3,
                    4 => egui::Key::Num4, 5 => egui::Key::Num5, 6 => egui::Key::Num6,
                    7 => egui::Key::Num7, 8 => egui::Key::Num8, 9 => egui::Key::Num9,
                    _ => egui::Key::Num0,
                }) && input.modifiers.ctrl
            }) {
                let _ = self.state.tab_manager.switch_to_tab(i);
            }
        }
    }

    fn show_save_dialog(&mut self, ctx: &Context) {
        let mut close_dialog = false;

        egui::Window::new("Save As")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Filename:");
                    let response = ui.text_edit_singleline(&mut self.save_filename);

                    if self.dialog_has_focus {
                        response.request_focus();
                    }

                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if !self.save_filename.is_empty() {
                            let _ = self.state.save_as(&self.save_filename);
                            close_dialog = true;
                        }
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        if !self.save_filename.is_empty() {
                            let _ = self.state.save_as(&self.save_filename);
                            close_dialog = true;
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        close_dialog = true;
                    }
                });

                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    close_dialog = true;
                }
            });

        if close_dialog {
            self.show_save_dialog = false;
            self.save_filename.clear();
            self.dialog_has_focus = false;
        }
    }

    fn show_shortcuts_window(&mut self, ctx: &Context) {
        egui::Window::new("Keyboard Shortcuts")
            .collapsible(true)
            .resizable(true)
            .show(ctx, |ui| {
                egui::Grid::new("shortcuts_grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Action");
                        ui.label("Shortcut");
                        ui.end_row();

                        let shortcuts = vec![
                            ("New", "Ctrl+N"), ("Save", "Ctrl+S"), ("Quit", "Ctrl+Q"),
                            ("Undo", "Ctrl+Z"), ("Redo", "Ctrl+Y"), ("Copy", "Ctrl+C"),
                            ("Cut", "Ctrl+X"), ("Paste", "Ctrl+V"), ("Find", "Ctrl+F"),
                            ("Select All", "Ctrl+A"), ("Switch Tab", "Ctrl+1-9"),
                        ];

                        for (action, shortcut) in shortcuts {
                            ui.label(action);
                            ui.label(shortcut);
                            ui.end_row();
                        }
                    });

                if ui.button("Close").clicked() {
                    self.show_shortcuts = false;
                }
            });
    }
}

impl eframe::App for QuickNotepadApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        themes::apply_theme(ctx);
        self.handle_shortcuts(ctx);
        self.menu_bar(ctx);
        self.status_bar(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            EditorPanel::new(&mut self.state, !self.dialog_has_focus).show(ui);
        });

        if self.show_save_dialog {
            self.show_save_dialog(ctx);
        }

        if self.show_shortcuts {
            self.show_shortcuts_window(ctx);
        }
    }
}