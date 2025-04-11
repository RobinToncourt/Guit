use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use egui::{TextStyle, FontId};
use egui_file::FileDialog;

use crate::Lang;
use crate::t;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Guit {
    font_size: f32,
    current_dir: PathBuf,
    #[serde(skip_serializing, skip_deserializing)]
    open_dir_dialog: Option<FileDialog>,
}

impl Default for Guit {
    fn default() -> Self {
        Self {
            font_size: 12.0,
            current_dir: PathBuf::new(),
            open_dir_dialog: None,
        }
    }
}

impl Guit {
    pub fn new(cc: &eframe::CreationContext<'_>, current_dir: PathBuf) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(
                storage,
                eframe::APP_KEY,
            ).unwrap_or_default();
        }

        configure_text_styles(&cc.egui_ctx);

        Self {
            current_dir,
            open_dir_dialog: None,
            ..Self::default()
        }
    }

    fn dir_dialog_selection(&mut self, ctx: &egui::Context) {
        if let Some(dialog) = &mut self.open_dir_dialog {
            if dialog.show(ctx).selected() {
                if let Some(folder) = dialog.path() {
                    self.current_dir = folder.to_path_buf();
                    let _ = std::env::set_current_dir(&self.current_dir);
                }
            }
        }
    }
}

fn configure_text_styles(ctx: &egui::Context) {
    let text_styles: BTreeMap<TextStyle, FontId> = [
        (TextStyle::Heading, FontId::proportional(25.0)),
        (TextStyle::Body, FontId::proportional(12.0)),
        (TextStyle::Monospace, FontId::monospace(12.0)),
        (TextStyle::Button, FontId::proportional(12.0)),
        (TextStyle::Small, FontId::proportional(8.0)),
    ]
    .into();

    ctx.all_styles_mut(move |style| style.text_styles = text_styles.clone());
}

impl eframe::App for Guit {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        resize_fonts(ctx, self.font_size);

        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.add(egui::Button::new(t!["open_folder"])).clicked() {
                    self.open_dir_dialog = Some(create_file_dialog(self.current_dir.clone()));
                }

                ui.separator();

                egui::widgets::global_theme_preference_buttons(ui);

                add_lang_selector(ui);

                add_font_size_drag_value(self, ui);
            });

            egui::menu::bar(ui, |ui| {
                ui.label("onglets");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.dir_dialog_selection(ctx);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
            });
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

#[inline(always)]
fn resize_fonts(ctx: &egui::Context, font_size: f32) {
    ctx.all_styles_mut(|style| {
        for (text_style, font_id) in &mut style.text_styles {
            match text_style {
                TextStyle::Heading => font_id.size = font_size + 13.0,
                TextStyle::Small => font_id.size = font_size - 4.0,
                _ => font_id.size = font_size,
            }
        }
    });
}

#[inline(always)]
fn create_file_dialog(current_dir: PathBuf) -> FileDialog {
    let filter = Box::new(|path: &Path| -> bool { path.is_dir() });
    let mut dialog = FileDialog::select_folder(Some(current_dir))
        .title(&t!["file_dialog_title"])
        .open_button_text(t!["file_dialog_open"].into())
        .cancel_button_text(t!["file_dialog_cancel"].into())
        .show_hidden_checkbox_text(t!["file_dialog_show_hidden"].into())
        .show_files_filter(filter)
        .show_rename(false)
        .show_new_folder(false)
        .multi_select(false);
    dialog.open();
    dialog
}

#[inline(always)]
fn add_lang_selector(ui: &mut egui::Ui) {
    ui.label(t!["lang_label"]);
    let lang: &mut Lang = &mut crate::LANG.lock().unwrap();
    egui::ComboBox::from_id_salt("lang-combobox")
    .selected_text(format!("{lang:?}"))
    .show_ui(ui, |ui| {
        ui.selectable_value(lang, Lang::Français, "Français");
        ui.selectable_value(lang, Lang::English, "English");
    });
}

#[inline(always)]
fn add_font_size_drag_value(guit: &mut Guit, ui: &mut egui::Ui) {
    ui.label(t!["fonts_size"]);
    if ui.add(egui::Button::new("-")).clicked()
        && guit.font_size > 8.0 {
            guit.font_size -= 1.0;
    }
    ui.add(
        egui::DragValue::new(&mut guit.font_size)
        .speed(1.0)
        .range(8.0..=20.0),
    );
    if ui.add(egui::Button::new("+")).clicked()
        && guit.font_size < 20.0 {
            guit.font_size += 1.0;
    }
}

#[inline(always)]
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
