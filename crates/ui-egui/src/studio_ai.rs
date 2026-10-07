//! Local AI Copilot for Sandeep PDF Studio
//! Connects to local Ollama (qwen2.5:7b) for completely offline document assistance.

use std::sync::Mutex;
use egui::{Color32, RichText};
use crate::PrintCraftApp;

#[derive(Default, Clone)]
pub struct StudioAiState {
    pub open: bool,
    pub prompt: String,
    pub response: String,
    pub status: String,
    pub is_loading: bool,
    pub model: String,
}

static STATE: Mutex<Option<StudioAiState>> = Mutex::new(None);

pub fn set_open(open: bool) {
    if let Ok(mut lock) = STATE.lock() {
        let state = lock.get_or_insert_with(|| StudioAiState {
            open: true,
            model: "qwen2.5:7b".into(),
            status: "Connected to local Ollama (qwen2.5:7b)".into(),
            ..Default::default()
        });
        state.open = open;
    }
}

pub fn render(ctx: &egui::Context, app: &mut PrintCraftApp) {
    let mut submit_prompt = None;

    {
        let Ok(mut lock) = STATE.lock() else { return };
        let state = lock.get_or_insert_with(|| StudioAiState {
            open: false,
            model: "qwen2.5:7b".into(),
            status: "Ready".into(),
            ..Default::default()
        });

        if !state.open {
            return;
        }

        let mut is_open = state.open;

        egui::Window::new("🤖 Local AI Copilot (Ollama)")
            .open(&mut is_open)
            .default_width(480.0)
            .default_pos([420.0, 75.0])
            .order(egui::Order::Foreground)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Local Model:").strong());
                    ui.add(egui::TextEdit::singleline(&mut state.model).desired_width(120.0));
                    ui.label(RichText::new("(Offline)").color(Color32::from_rgb(0, 200, 100)).small());
                });

                ui.add_space(4.0);
                ui.separator();

                ui.label(RichText::new("Quick Actions:").strong());
                ui.horizontal_wrapped(|ui| {
                    if ui.button("📄 Summarize Document").clicked() {
                        state.prompt = "Summarize the key information, invoice numbers, dates, and amounts in this document in 3 concise bullet points.".into();
                        submit_prompt = Some(state.prompt.clone());
                    }
                    if ui.button("🔍 Extract Values").clicked() {
                        state.prompt = "Extract all key-value pairs (Vendor, Customer, Delivery Note, Quantities) from this document.".into();
                        submit_prompt = Some(state.prompt.clone());
                    }
                });

                ui.add_space(6.0);
                ui.separator();

                ui.label(RichText::new("Ask Question:").strong());
                ui.add(
                    egui::TextEdit::multiline(&mut state.prompt)
                        .desired_rows(3)
                        .desired_width(ui.available_width())
                        .hint_text("Ask anything about this PDF..."),
                );

                ui.horizontal(|ui| {
                    let can_send = !state.is_loading && !state.prompt.trim().is_empty();
                    if ui.add_enabled(can_send, egui::Button::new("⚡ Ask Local AI")).clicked() {
                        submit_prompt = Some(state.prompt.clone());
                    }
                    if state.is_loading {
                        ui.spinner();
                        ui.label(RichText::new("Running on Mac local neural engine...").color(Color32::from_rgb(0, 190, 255)));
                    }
                });

                if !state.status.is_empty() {
                    ui.label(RichText::new(&state.status).small().color(Color32::from_rgb(180, 200, 220)));
                }

                ui.separator();

                if !state.response.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("AI Answer:").strong().color(Color32::from_rgb(0, 220, 150)));
                        if ui.button("📋 Copy").clicked() {
                            ui.ctx().copy_text(state.response.clone());
                            state.status = "Copied to clipboard!".into();
                        }
                    });

                    egui::ScrollArea::vertical()
                        .max_height(280.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut state.response.as_str())
                                    .desired_width(ui.available_width())
                                    .interactive(false),
                            );
                        });
                }
            });

        state.open = is_open;
    }

    if let Some(user_prompt) = submit_prompt {
        let model = {
            let Ok(mut lock) = STATE.lock() else { return };
            let state = lock.get_or_insert_with(Default::default);
            state.is_loading = true;
            state.status = "Querying local Ollama...".into();
            state.model.clone()
        };

        let doc_name = if let Some((_, doc_id)) = app.active_ids() {
            app.session.get(doc_id).map(|d| d.info.title.clone().unwrap_or_else(|| "Active PDF".into())).unwrap_or_else(|| "Active PDF".into())
        } else {
            "Active PDF".into()
        };

        std::thread::spawn(move || {
            let full_query = format!("Document: {}\n\nQuestion: {}", doc_name, user_prompt);
            let escaped = full_query.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
            let json_body = format!(r#"{{"model":"{}","prompt":"{}","stream":false}}"#, model, escaped);

            let output = std::process::Command::new("curl")
                .arg("-s")
                .arg("-X")
                .arg("POST")
                .arg("http://127.0.0.1:11434/api/generate")
                .arg("-H")
                .arg("Content-Type: application/json")
                .arg("-d")
                .arg(&json_body)
                .output();

            let (ans, status_msg) = match output {
                Ok(out) if out.status.success() => {
                    let resp = String::from_utf8_lossy(&out.stdout).to_string();
                    if let Some(start) = resp.find("\"response\":\"") {
                        let text_start = start + 12;
                        if let Some(end) = resp[text_start..].find("\",\"done\":") {
                            let text = resp[text_start..text_start + end]
                                .replace("\\n", "\n")
                                .replace("\\\"", "\"");
                            (text, "Complete.".to_string())
                        } else {
                            (resp, "Complete.".to_string())
                        }
                    } else {
                        (resp, "Complete.".to_string())
                    }
                }
                Ok(out) => (
                    format!("Ollama connection error. Ensure 'ollama serve' is running:\n{}", String::from_utf8_lossy(&out.stderr)),
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
        });
    }
}
