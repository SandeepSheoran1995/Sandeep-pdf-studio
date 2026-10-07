//! In-Place QR Code Reader, Inspector & Direct Page Replacer
//! Defaults: 100.0% scale, 0.0 pt border, Foreground layer.

use std::sync::Mutex;
use egui::{Color32, RichText};
use qrcode::QrCode;
use image::{ImageBuffer, Luma};
use rqrr::PreparedImage;

#[derive(Default, Clone, Debug)]
pub struct DetectedQr {
    pub text: String,
    pub rect_pts: [f64; 4],
    pub page: usize,
}

pub struct QrState {
    pub open: bool,
    pub detected_codes: Vec<DetectedQr>,
    pub selected_index: Option<usize>,
    pub original_text: String,
    pub edited_text: String,
    pub status: String,
    pub is_scanning: bool,
    pub scale_percent: f64,
    pub mask_pad: f64,
    pub nudge_x: f64,
    pub nudge_y: f64,
}

impl Default for QrState {
    fn default() -> Self {
        Self {
            open: false,
            detected_codes: Vec::new(),
            selected_index: None,
            original_text: String::new(),
            edited_text: String::new(),
            status: String::new(),
            is_scanning: false,
            scale_percent: 100.0,
            mask_pad: 0.0,
            nudge_x: 0.0,
            nudge_y: 0.0,
        }
    }
}

static STATE: Mutex<Option<QrState>> = Mutex::new(None);

pub fn set_open(open: bool) {
    if let Ok(mut lock) = STATE.lock() {
        let state = lock.get_or_insert_with(|| QrState {
            open: true,
            status: "Click 'Scan Active Page' to detect existing QR codes.".into(),
            scale_percent: 100.0,
            mask_pad: 0.0,
            ..Default::default()
        });
        state.open = open;
        if open {
            state.scale_percent = 100.0;
            state.mask_pad = 0.0;
        }
    }
}

pub fn render(ctx: &egui::Context, app: &mut crate::PrintCraftApp) {
    let mut scan_requested = false;
    let mut apply_replacement = None;

    {
        let Ok(mut lock) = STATE.lock() else { return };
        let state = lock.get_or_insert_with(|| QrState {
            open: false,
            scale_percent: 100.0,
            mask_pad: 0.0,
            ..Default::default()
        });

        if !state.open {
            return;
        }

        let mut is_open = state.open;

        egui::Window::new("🔄 In-Place QR Code Inspector & Updater")
            .open(&mut is_open)
            .default_width(520.0)
            .default_pos([420.0, 75.0])
            .order(egui::Order::Foreground)
            .resizable(true)
            .show(ctx, |ui| {
                ui.label(RichText::new("1. Scan & Read QR Code").strong());
                ui.horizontal(|ui| {
                    if ui.button("🔍 Scan Active Page for QR Codes").clicked() {
                        scan_requested = true;
                    }
                    if state.is_scanning {
                        ui.spinner();
                        ui.label("Decoding QR codes on page...");
                    }
                });

                ui.add_space(4.0);
                ui.separator();

                if !state.detected_codes.is_empty() {
                    ui.label(RichText::new(format!("Detected {} QR Code(s):", state.detected_codes.len())).strong().color(Color32::from_rgb(0, 190, 80)));
                    for (i, qr) in state.detected_codes.iter().enumerate() {
                        let is_sel = state.selected_index == Some(i);
                        let label = format!("QR #{}: {} ({:.1}×{:.1} pt)", i + 1, truncate_str(&qr.text, 28), qr.rect_pts[2], qr.rect_pts[3]);
                        if ui.selectable_label(is_sel, label).clicked() {
                            state.selected_index = Some(i);
                            state.original_text = qr.text.clone();
                            state.edited_text = qr.text.clone();
                            state.status = "QR Code selected. Edit payload below.".into();
                        }
                    }
                    ui.add_space(4.0);
                }

                if let Some(idx) = state.selected_index {
                    if let Some(qr) = state.detected_codes.get(idx) {
                        ui.label(RichText::new("2. Decoded Payload").strong());
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut state.original_text)
                                    .desired_width(ui.available_width() - 80.0)
                                    .interactive(false),
                            );
                            if ui.button("📋 Copy").clicked() {
                                ctx.copy_text(state.original_text.clone());
                                state.status = "Copied QR payload to clipboard!".into();
                            }
                        });

                        ui.add_space(6.0);
                        ui.label(RichText::new("3. Edit / Modify Data:").strong());
                        ui.add(
                            egui::TextEdit::multiline(&mut state.edited_text)
                                .desired_rows(3)
                                .desired_width(ui.available_width())
                                .hint_text("Enter new or modified URL / text data..."),
                        );

                        ui.add_space(6.0);
                        ui.separator();

                        let raw_side = (qr.rect_pts[2] + qr.rect_pts[3]) / 2.0;
                        let target_side = raw_side * (state.scale_percent / 100.0);

                        ui.label(RichText::new("4. Size & Border Settings").strong());
                        ui.horizontal(|ui| {
                            ui.label(format!("Scale: {:.1}%", state.scale_percent));
                            if ui.button("[-2%]").clicked() { state.scale_percent -= 2.0; }
                            if ui.button("[100% Fit]").clicked() { state.scale_percent = 100.0; }
                            if ui.button("[+2%]").clicked() { state.scale_percent += 2.0; }
                        });
                        ui.add(egui::Slider::new(&mut state.scale_percent, 75.0..=125.0).text("% scale"));

                        ui.horizontal(|ui| {
                            ui.label("Border Padding:");
                            if ui.button("[0 pt]").clicked() { state.mask_pad = 0.0; }
                            if ui.button("[1 pt]").clicked() { state.mask_pad = 1.0; }
                            ui.add(egui::Slider::new(&mut state.mask_pad, 0.0..=5.0).text("pt"));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Position Nudge:");
                            if ui.button("←").clicked() { state.nudge_x -= 0.5; }
                            if ui.button("→").clicked() { state.nudge_x += 0.5; }
                            if ui.button("↓").clicked() { state.nudge_y -= 0.5; }
                            if ui.button("↑").clicked() { state.nudge_y += 0.5; }
                            if ui.button("Reset").clicked() {
                                state.nudge_x = 0.0;
                                state.nudge_y = 0.0;
                            }
                            ui.label(format!("({:+.1}, {:+.1} pt)", state.nudge_x, state.nudge_y));
                        });

                        ui.label(RichText::new(format!("Original: {:.1} pt  ➔  New QR: {:.1} pt (Border: {:.1} pt)", raw_side, target_side, state.mask_pad)).small().color(Color32::from_rgb(180, 220, 255)));

                        ui.add_space(8.0);
                        if ui.button("⚡ Replace QR Code In-Place Directly On Page").clicked() {
                            if state.edited_text.trim().is_empty() {
                                state.status = "Data cannot be empty.".into();
                            } else {
                                apply_replacement = Some((
                                    qr.clone(),
                                    state.edited_text.clone(),
                                    state.scale_percent,
                                    state.mask_pad,
                                    state.nudge_x,
                                    state.nudge_y,
                                ));
                            }
                        }
                    }
                }

                if !state.status.is_empty() {
                    ui.add_space(4.0);
                    ui.separator();
                    ui.label(RichText::new(&state.status).color(Color32::from_rgb(0, 180, 220)));
                }
            });

        state.open = is_open;
    }

    if scan_requested {
        let mut detected = Vec::new();
        if let Some((view_idx, doc_id)) = app.active_ids() {
            if let Some(view) = app.views.get(view_idx) {
                let current_page = view.current;
                if let Some(doc) = app.session.get(doc_id) {
                    if let Some(page_info) = doc.info.pages.get(current_page) {
                        let page_w = page_info.width as f64;
                        let page_h = page_info.height as f64;

                        let dpi = 150.0;
                        let mut exporter = printcraft_engine::export::Exporter::new(doc);
                        let export_res: Result<Vec<u8>, _> = exporter.image(current_page, dpi, printcraft_engine::export::ImageFormat::Png);
                        if let Ok(png_bytes) = export_res {
                            if let Ok(dyn_img) = image::load_from_memory(&png_bytes) {
                                let gray = dyn_img.to_luma8();
                                let (img_w, img_h) = (gray.width() as usize, gray.height() as usize);
                                let mut prepared = PreparedImage::prepare_from_greyscale(img_w, img_h, |x, y| {
                                    gray.get_pixel(x as u32, y as u32).0[0]
                                });
                                let grids = prepared.detect_grids();
                                for grid in grids {
                                    if let Ok((_meta, content)) = grid.decode() {
                                        let min_x = grid.bounds.iter().map(|p| p.x).min().unwrap_or(0);
                                        let max_x = grid.bounds.iter().map(|p| p.x).max().unwrap_or(img_w as i32);
                                        let min_y = grid.bounds.iter().map(|p| p.y).min().unwrap_or(0);
                                        let max_y = grid.bounds.iter().map(|p| p.y).max().unwrap_or(img_h as i32);

                                        let scale_x = page_w / (img_w as f64);
                                        let scale_y = page_h / (img_h as f64);

                                        let pdf_x0 = (min_x as f64) * scale_x;
                                        let pdf_x1 = (max_x as f64) * scale_x;
                                        let pdf_y0 = page_h - ((max_y as f64) * scale_y);
                                        let pdf_y1 = page_h - ((min_y as f64) * scale_y);

                                        let pt_w = pdf_x1 - pdf_x0;
                                        let pt_h = pdf_y1 - pdf_y0;

                                        detected.push(DetectedQr {
                                            text: content,
                                            rect_pts: [pdf_x0, pdf_y0, pt_w, pt_h],
                                            page: current_page,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Ok(mut lock) = STATE.lock() {
            if let Some(ref mut state) = *lock {
                state.is_scanning = false;
                state.detected_codes = detected;
                if state.detected_codes.is_empty() {
                    state.status = "No QR codes found on active page.".into();
                    state.selected_index = None;
                } else {
                    state.status = format!("Found {} QR code(s). Selected.", state.detected_codes.len());
                    state.selected_index = Some(0);
                    state.original_text = state.detected_codes[0].text.clone();
                    state.edited_text = state.detected_codes[0].text.clone();
                }
            }
        }
    }

    if let Some((qr_info, new_payload, scale_percent, mask_pad, nudge_x, nudge_y)) = apply_replacement {
        match QrCode::new(new_payload.as_bytes()) {
            Ok(code) => {
                let raw_qr: ImageBuffer<Luma<u8>, Vec<u8>> = code
                    .render::<Luma<u8>>()
                    .quiet_zone(false)
                    .build();

                let qr_w = raw_qr.width();
                let qr_h = raw_qr.height();

                let margin_px = if mask_pad <= 0.001 {
                    0u32
                } else {
                    ((qr_w as f64) * (mask_pad / 30.0)).round().max(1.0) as u32
                };

                let mut png_bytes = Vec::new();
                let encode_ok = if margin_px == 0 {
                    let mut cursor = std::io::Cursor::new(&mut png_bytes);
                    raw_qr.write_to(&mut cursor, image::ImageFormat::Png).is_ok()
                } else {
                    let canvas_w = qr_w + margin_px * 2;
                    let canvas_h = qr_h + margin_px * 2;
                    let mut composite = ImageBuffer::from_pixel(canvas_w, canvas_h, Luma([255u8]));
                    for y in 0..qr_h {
                        for x in 0..qr_w {
                            composite.put_pixel(x + margin_px, y + margin_px, *raw_qr.get_pixel(x, y));
                        }
                    }
                    let mut cursor = std::io::Cursor::new(&mut png_bytes);
                    composite.write_to(&mut cursor, image::ImageFormat::Png).is_ok()
                };

                if encode_ok {
                    let raw_w = qr_info.rect_pts[2];
                    let raw_h = qr_info.rect_pts[3];
                    let center_x = qr_info.rect_pts[0] + raw_w / 2.0;
                    let center_y = qr_info.rect_pts[1] + raw_h / 2.0;

                    let base_side = (raw_w + raw_h) / 2.0;
                    let code_side = base_side * (scale_percent / 100.0);
                    let total_coverage = code_side + mask_pad * 2.0;

                    let x0 = center_x - total_coverage / 2.0 + nudge_x;
                    let y0 = center_y - total_coverage / 2.0 + nudge_y;
                    let x1 = x0 + total_coverage;
                    let y1 = y0 + total_coverage;

                    let rect = [x0, y0, x1, y1];

                    let edit = printcraft_engine::Edit::AddCustomStamp {
                        page: qr_info.page,
                        rect,
                        name: "Updated QR Code".into(),
                        file: printcraft_engine::MarkFile {
                            name: "updated_qr.png".into(),
                            bytes: std::sync::Arc::new(png_bytes),
                            page: 0,
                        },
                        author: "Sandeep PDF Studio".into(),
                    };
                    app.apply_edit(edit);

                    app.left_open = false;
                    app.notify(format!("✔ QR Code on Page {} replaced in-place ({:.1} pt)!", qr_info.page + 1, code_side));
                    if let Ok(mut lock) = STATE.lock() {
                        if let Some(ref mut state) = *lock {
                            state.status = format!("✔ Successfully replaced QR on Page {}! Scale: {:.0}%, Border: {:.1} pt.", qr_info.page + 1, scale_percent, mask_pad);
                        }
                    }
                }
            }
            Err(e) => {
                if let Ok(mut lock) = STATE.lock() {
                    if let Some(ref mut state) = *lock {
                        state.status = format!("Encoding error: {}", e);
                    }
                }
            }
        }
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len])
    } else {
        s.to_string()
    }
}
