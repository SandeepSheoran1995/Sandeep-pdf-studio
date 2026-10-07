//! Google Gemini Cloud AI Integration for Sandeep PDF Studio
//! Direct cloud connection to Gemini 2.0/1.5/2.5 Flash with persistent key and model storage.

use std::sync::Mutex;
use egui::{Color32, RichText};
use crate::PrintCraftApp;

#[derive(Default, Clone)]
pub struct GeminiState {
    pub open: bool,
    pub api_key: String,
    pub key_input: String,
    pub model: String,
    pub is_editing_key: bool,
    pub prompt: String,
    pub response: String,
    pub status: String,
    pub is_loading: bool,
}

static STATE: Mutex<Option<GeminiState>> = Mutex::new(None);

fn get_config_dir() -> std::path::PathBuf {
    let mut path = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    path.push(".config");
    path.push("sandeep-pdf-studio");
    std::fs::create_dir_all(&path).ok();
    path
}

fn load_saved_key() -> String {
    let mut p = get_config_dir();
    p.push("gemini.key");
    std::fs::read_to_string(p).unwrap_or_default().trim().trim_matches('"').trim_matches('\'').trim().to_string()
}

fn save_key(key: &str) {
    let mut p = get_config_dir();
    p.push("gemini.key");
    std::fs::write(p, key.trim().trim_matches('"').trim_matches('\'').trim()).ok();
}

fn load_saved_model() -> String {
    let mut p = get_config_dir();
    p.push("gemini.model");
    let m = std::fs::read_to_string(p).unwrap_or_default().trim().to_string();
    if m.is_empty() {
        "gemini-2.0-flash".to_string()
    } else {
        m
    }
}

fn save_model(model: &str) {
    let mut p = get_config_dir();
    p.push("gemini.model");
    std::fs::write(p, model.trim()).ok();
}

pub fn set_open(open: bool) {
    if let Ok(mut lock) = STATE.lock() {
        let state = lock.get_or_insert_with(|| {
            let saved_key = load_saved_key();
            let saved_model = load_saved_model();
            let has_key = !saved_key.is_empty();
            GeminiState {
                open: true,
                api_key: saved_key.clone(),
                key_input: saved_key,
                model: saved_model,
                is_editing_key: !has_key,
                status: if has_key { "Ready. Connected to Google Gemini.".into() } else { "Please configure Gemini API key.".into() },
                ..Default::default()
            }
        });
        state.open = open;
    }
}

pub fn render(ctx: &egui::Context, app: &mut PrintCraftApp) {
    let Ok(mut lock) = STATE.lock() else { return };
    let state = lock.get_or_insert_with(|| {
        let saved_key = load_saved_key();
        let saved_model = load_saved_model();
        let has_key = !saved_key.is_empty();
        GeminiState {
            open: false,
            api_key: saved_key.clone(),
            key_input: saved_key,
            model: saved_model,
            is_editing_key: !has_key,
            status: if has_key { "Ready. Connected to Google Gemini.".into() } else { "Please configure Gemini API key.".into() },
            ..Default::default()
        }
    });

    if !state.open {
        return;
    }

    let mut is_open = state.open;
    drop(lock);

    egui::Window::new("✨ Google Gemini Cloud AI")
        .open(&mut is_open)
        .default_width(520.0)
        .default_pos([420.0, 75.0])
        .order(egui::Order::Foreground)
        .resizable(true)
        .show(ctx, |ui| {
            render_inner(ui, app);
        });

    if let Ok(mut lock) = STATE.lock() {
        if let Some(ref mut s) = *lock {
            s.open = is_open;
        }
    }
}

pub fn render_panel(ui: &mut egui::Ui, app: &mut PrintCraftApp) {
    render_inner(ui, app);
}

fn render_inner(ui: &mut egui::Ui, app: &mut PrintCraftApp) {
    let mut submit_prompt = None;

    {
        let Ok(mut lock) = STATE.lock() else { return };
        let state = lock.get_or_insert_with(|| {
            let saved_key = load_saved_key();
            let saved_model = load_saved_model();
            GeminiState {
                api_key: saved_key.clone(),
                key_input: saved_key,
                model: saved_model,
                ..Default::default()
            }
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("✨ Google Gemini Cloud AI").strong().color(Color32::from_rgb(0, 190, 255)).size(15.0));
            if !state.is_editing_key && ui.small_button("⚙️ Key").clicked() {
                state.is_editing_key = true;
            }
        });
        ui.add_space(2.0);

        // Model Selector
        ui.horizontal(|ui| {
            let prev_model = state.model.clone();
            ui.label(RichText::new("Model:").strong());
            egui::ComboBox::from_label("")
                .selected_text(&state.model)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.model, "gemini-2.0-flash".into(), "gemini-2.0-flash (Recommended)");
                    ui.selectable_value(&mut state.model, "gemini-1.5-flash".into(), "gemini-1.5-flash (High Capacity)");
                    ui.selectable_value(&mut state.model, "gemini-2.5-flash".into(), "gemini-2.5-flash");
                    ui.selectable_value(&mut state.model, "gemini-1.5-pro".into(), "gemini-1.5-pro");
                });
            if prev_model != state.model {
                save_model(&state.model);
            }
        });

        ui.horizontal(|ui| {
            ui.label(RichText::new("Custom Model:").small());
            let resp = ui.add(egui::TextEdit::singleline(&mut state.model).desired_width(170.0));
            if resp.lost_focus() {
                save_model(&state.model);
            }
        });

        ui.add_space(2.0);

        // API Key Section
        if state.is_editing_key || state.api_key.is_empty() {
            ui.group(|ui| {
                ui.label(RichText::new("Google Gemini API Key").strong());
                ui.label(RichText::new("Free key from aistudio.google.com").small().color(Color32::GRAY));
                ui.add_space(2.0);
                ui.add(
                    egui::TextEdit::singleline(&mut state.key_input)
                        .password(true)
                        .desired_width(ui.available_width() - 10.0)
                        .hint_text("Paste API key here"),
                );
                ui.horizontal(|ui| {
                    if ui.button("💾 Save Key").clicked() {
                        let trimmed = state.key_input.trim().trim_matches('"').trim_matches('\'').trim().to_string();
                        if !trimmed.is_empty() {
                            save_key(&trimmed);
                            state.api_key = trimmed;
                            state.is_editing_key = false;
                            state.status = "✔ Gemini API Key saved!".into();
                        }
                    }
                    if !state.api_key.is_empty() && ui.button("Cancel").clicked() {
                        state.is_editing_key = false;
                    }
                });
            });
            ui.separator();
        }

        // Quick Document Actions
        ui.label(RichText::new("Quick Document Actions").strong());
        ui.horizontal_wrapped(|ui| {
            if ui.button("📄 Analyze Page").clicked() {
                state.prompt = "Provide a structured breakdown of the active document page. Identify invoice numbers, dates, customer names, quantities, and important values.".into();
                submit_prompt = Some(state.prompt.clone());
            }
            if ui.button("📝 Summarize").clicked() {
                state.prompt = "Summarize the primary purpose and key details of this page in 3 clear bullet points.".into();
                submit_prompt = Some(state.prompt.clone());
            }
            if ui.button("🔍 Extract Tables").clicked() {
                state.prompt = "Extract all tabular and reference data into a clean plain text layout.".into();
                submit_prompt = Some(state.prompt.clone());
            }
        });

        ui.add_space(6.0);
        ui.separator();

        // Custom Question Box
        ui.label(RichText::new("Ask Gemini Anything:").strong());
        ui.add(
            egui::TextEdit::multiline(&mut state.prompt)
                .desired_rows(3)
                .desired_width(ui.available_width())
                .hint_text("Ask a question about this PDF..."),
        );

        ui.horizontal(|ui| {
            let can_send = !state.is_loading && !state.api_key.is_empty() && !state.prompt.trim().is_empty();
            if ui.add_enabled(can_send, egui::Button::new("🚀 Ask Gemini")).clicked() {
                submit_prompt = Some(state.prompt.clone());
            }
            if state.is_loading {
                ui.spinner();
                ui.label(RichText::new(format!("Querying {} ...", state.model)).color(Color32::from_rgb(0, 190, 255)));
            }
        });

        if !state.status.is_empty() {
            ui.label(RichText::new(&state.status).small().color(Color32::from_rgb(180, 200, 220)));
        }

        ui.separator();

        // Response Section
        if !state.response.is_empty() {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Gemini Response:").strong().color(Color32::from_rgb(0, 220, 150)));
                if ui.button("📋 Copy").clicked() {
                    ui.ctx().copy_text(state.response.clone());
                    state.status = "Copied response to clipboard!".into();
                }
            });

            egui::ScrollArea::vertical()
                .max_height(280.0)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut state.response.as_str())
                            .desired_width(ui.available_width())
                            .interactive(false),
                    );
                });
        }

        if state.is_loading {
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
        }
    }

    if let Some(user_prompt) = submit_prompt {
        let (clean_key, clean_model) = {
            let Ok(mut lock) = STATE.lock() else { return };
            let state = lock.get_or_insert_with(Default::default);
            state.is_loading = true;
            state.status = format!("Connecting to Google Gemini ({})...", state.model);
            (
                state.api_key.trim().trim_matches('"').trim_matches('\'').trim().to_string(),
                state.model.trim().to_string(),
            )
        };

        let doc_name = if let Some((_, doc_id)) = app.active_ids() {
            app.session.get(doc_id).map(|d| d.info.title.clone().unwrap_or_else(|| "Active PDF Document".into())).unwrap_or_else(|| "Active PDF Document".into())
        } else {
            "Active PDF Document".into()
        };

        let ctx_clone = ui.ctx().clone();

        std::thread::spawn(move || {
            let full_query = format!(
                "You are an expert AI document assistant analyzing: '{}'.\n\nUser Question:\n{}",
                doc_name, user_prompt
            );

            let escaped = full_query
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\r', "\\r")
                .replace('\t', "\\t");

            let json_body = format!(
                r#"{{"contents":[{{"parts":[{{"text":"{}"}}]}}]}}"#,
                escaped
            );

            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
                clean_model
            );

            let mut child = match std::process::Command::new("curl")
                .arg("-4")
                .arg("-s")
                .arg("-S")
                .arg("--max-time")
                .arg("60")
                .arg("--connect-timeout")
                .arg("10")
                .arg("-X")
                .arg("POST")
                .arg("-H")
                .arg("Content-Type: application/json")
                .arg("-H")
                .arg(format!("x-goog-api-key: {}", clean_key))
                .arg("--data-binary")
                .arg("@-")
                .arg(&url)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    if let Ok(mut lock) = STATE.lock() {
                        if let Some(ref mut state) = *lock {
                            state.response = format!("Failed to spawn curl: {}", e);
                            state.status = "Failed".to_string();
                            state.is_loading = false;
                        }
                    }
                    ctx_clone.request_repaint();
                    return;
                }
            };

            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                let _ = stdin.write_all(json_body.as_bytes());
            }

            let (ans, status_msg) = match child.wait_with_output() {
                Ok(out) if out.status.success() => {
                    let resp_str = String::from_utf8_lossy(&out.stdout).to_string();
                    if resp_str.contains("503") || resp_str.contains("high demand") || resp_str.contains("UNAVAILABLE") {
                        let fallback_model = if clean_model != "gemini-2.0-flash" { "gemini-2.0-flash" } else { "gemini-1.5-flash" };
                        let fb_url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", fallback_model);
                        
                        let fb_child = std::process::Command::new("curl")
                            .arg("-4").arg("-s").arg("-S").arg("--max-time").arg("45")
                            .arg("-X").arg("POST")
                            .arg("-H").arg("Content-Type: application/json")
                            .arg("-H").arg(format!("x-goog-api-key: {}", clean_key))
                            .arg("--data-binary").arg("@-")
                            .arg(&fb_url)
                            .stdin(std::process::Stdio::piped())
                            .stdout(std::process::Stdio::piped())
                            .stderr(std::process::Stdio::piped())
                            .spawn();

                        if let Ok(mut fb_c) = fb_child {
                            if let Some(mut fb_stdin) = fb_c.stdin.take() {
                                use std::io::Write;
                                let _ = fb_stdin.write_all(json_body.as_bytes());
                            }
                            if let Ok(fb_out) = fb_c.wait_with_output() {
                                let fb_resp = String::from_utf8_lossy(&fb_out.stdout).to_string();
                                if let Some(fb_text) = extract_gemini_text(&fb_resp) {
                                    (format!("[Notice: Primary model experienced high demand (503). Auto-switched to {}]\n\n{}", fallback_model, fb_text), "Complete.".to_string())
                                } else {
                                    (format!("Primary model busy (503). Fallback response:\n{}", fb_resp), "High demand".to_string())
                                }
                            } else {
                                ("Google API 503 (High demand). Please try again or select another model.".to_string(), "Busy".to_string())
                            }
                        } else {
                            ("Google API 503 (High demand). Please try again or select another model.".to_string(), "Busy".to_string())
                        }
                    } else if let Some(text) = extract_gemini_text(&resp_str) {
                        (text, "Complete.".to_string())
                    } else if resp_str.contains("error") {
                        (format!("Google API Error:\n{}", resp_str), "Error".to_string())
                    } else {
                        (resp_str, "Complete.".to_string())
                    }
                }
                Ok(out) => (
                    format!("Curl error (Code {}):\n{}", out.status.code().unwrap_or(-1), String::from_utf8_lossy(&out.stderr)),
                    "Failed".to_string(),
                ),
                Err(e) => (format!("Network error: {}", e), "Failed".to_string()),
            };

            if let Ok(mut lock) = STATE.lock() {
                if let Some(ref mut state) = *lock {
                    state.response = ans;
                    state.status = status_msg;
                    state.is_loading = false;
                }
            }

            ctx_clone.request_repaint();
        });
    }
}

fn extract_gemini_text(json: &str) -> Option<String> {
    let marker = "\"text\": \"";
    let start = json.find(marker)? + marker.len();
    let rest = &json[start..];

    let mut result = String::new();
    let mut chars = rest.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '"' {
            break;
        } else if c == '\\' {
            if let Some(next) = chars.next() {
                match next {
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    _ => {
                        result.push('\\');
                        result.push(next);
                    }
                }
            }
        } else {
            result.push(c);
        }
    }
    Some(result)
}
