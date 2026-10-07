//! Persistent Studio Tools Palette for Sandeep PDF Studio
use std::sync::Mutex;
use egui::RichText;
use crate::{PrintCraftApp, RightPanel};

static OPEN: Mutex<bool> = Mutex::new(false);

pub fn set_open(open: bool) {
    if let Ok(mut lock) = OPEN.lock() {
        *lock = open;
    }
}

pub fn render(ctx: &egui::Context, app: &mut PrintCraftApp) {
    let mut is_open = OPEN.lock().map(|l| *l).unwrap_or(false);
    if !is_open {
        return;
    }

    egui::Window::new("🛠️ Studio Tools Palette")
        .open(&mut is_open)
        .default_width(340.0)
        .default_pos([450.0, 80.0])
        .order(egui::Order::Foreground)
        .resizable(true)
        .show(ctx, |ui| {
            ui.label(RichText::new("AI Document Intelligence").strong());
            if ui.button("🤖 Ask Document (Local AI)").clicked() {
                crate::studio_ai::set_open(true);
            }
            if ui.button("✨ Ask Google Gemini AI").clicked() {
                crate::gemini_ai::set_open(true);
            }
            if ui.button("📊 Compare Two Documents").clicked() {
                app.right = Some(RightPanel::Compare);
            }
            ui.separator();

            ui.label(RichText::new("QR Code Studio").strong());
            if ui.button("🔄 Update QR Code (In-Place)").clicked() {
                crate::qr_updater::set_open(true);
            }
            ui.separator();

            ui.label(RichText::new("High-Res Page Exporters").strong());
            if ui.button("🖼️ Export Pages as PNG (150 DPI - Web)").clicked() {
                app.export_draft.dpi = 150.0;
                app.export_draft.format = printcraft_engine::export::ImageFormat::Png;
                app.start_export(crate::export_ui::ExportKind::Image);
            }
            if ui.button("🖼️ Export Pages as PNG (300 DPI - Print)").clicked() {
                app.export_draft.dpi = 300.0;
                app.export_draft.format = printcraft_engine::export::ImageFormat::Png;
                app.start_export(crate::export_ui::ExportKind::Image);
            }
            if ui.button("🖼️ Export Pages as JPEG (300 DPI)").clicked() {
                app.export_draft.dpi = 300.0;
                app.export_draft.format = printcraft_engine::export::ImageFormat::Jpeg { quality: 90 };
                app.start_export(crate::export_ui::ExportKind::Image);
            }
            ui.separator();

            ui.label(RichText::new("Stamps, Sign & Legal").strong());
            if ui.button("📑 Open Stamp Drawer").clicked() {
                app.left = crate::LeftPanel::Tool("stamp");
                app.left_open = true;
            }
            if ui.button("✍️ Place Signature Area").clicked() {
                app.quick_tool = crate::QuickTool::SignArea { certify: false };
                app.notify("Drag on page to place signature area.");
            }
            if ui.button("💧 Watermark Studio").clicked() {
                app.execute("edit.watermark");
            }
            if ui.button("🔢 Bates Legal Stamping").clicked() {
                app.execute("edit.bates");
            }
            if ui.button("📄 Headers & Footers").clicked() {
                app.execute("edit.header_footer");
            }
            ui.separator();

            ui.label(RichText::new("Security & Privacy").strong());
            if ui.button("🧹 Sanitize & Strip Metadata").clicked() {
                app.execute("redact.sanitize");
            }
            if ui.button("🔒 Password & Permissions").clicked() {
                app.execute("protect.password");
            }
            if ui.button("👁️ Remove Hidden Data").clicked() {
                app.execute("protect.remove_hidden");
            }
            ui.separator();

            ui.label(RichText::new("OCR & Optimization").strong());
            if ui.button("🔍 OCR Text Recognition").clicked() {
                app.execute("ocr.recognize");
            }
            if ui.button("⚡ Reduce File Size (Quick)").clicked() {
                app.execute("optimize.reduce");
            }
            if ui.button("⚙️ Advanced PDF Optimizer").clicked() {
                app.execute("optimize.advanced");
            }
        });

    if let Ok(mut lock) = OPEN.lock() {
        *lock = is_open;
    }
}
