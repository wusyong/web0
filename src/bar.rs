use crate::Panel;
use eframe::{egui, epi};
use std::sync::mpsc::Receiver;

#[derive(Debug, PartialEq, Copy, Clone)]
enum Method {
    Get,
    Post,
}

/// A top panel bar to show url input field and other configurations.
#[derive(Debug)]
pub struct TopBar {
    url: String,
    method: Method,
    request_body: String,
    pub in_progress: Option<Receiver<Result<ureq::Response, ureq::Error>>>,
}

impl Default for TopBar {
    fn default() -> Self {
        Self {
            url: String::new(),
            method: Method::Get,
            request_body: String::new(),
            in_progress: Default::default(),
        }
    }
}

impl Panel for TopBar {
    fn name(&self) -> &'static str {
        "Top Panel Bar"
    }
    fn show(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        let mut trigger_fetch = false;
        egui::TopBottomPanel::top("Bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("URL:");
                trigger_fetch |= ui
                    .add(egui::TextEdit::singleline(&mut self.url).desired_width(f32::INFINITY))
                    .lost_focus();
            });

            ui.horizontal(|ui| {
                ui.label("Method:");
                ui.selectable_value(&mut self.method, Method::Get, "GET")
                    .clicked();
                ui.selectable_value(&mut self.method, Method::Post, "POST")
                    .clicked();
            });

            if self.method == Method::Post {
                ui.horizontal(|ui| {
                    ui.label("POST Body:");
                    trigger_fetch |= ui
                        .add(
                            egui::TextEdit::multiline(&mut self.request_body)
                                .code_editor()
                                .desired_rows(1),
                        )
                        .lost_focus();
                });
            }

            ui.horizontal(|ui| {
                if ui.button("Random image").clicked() {
                    let seed = ui.input().time;
                    let side = 640;
                    self.url = format!("https://picsum.photos/seed/{}/{}", seed, side);
                    trigger_fetch = true;
                }

                if ui.button("POST to httpbin.org").clicked() {
                    self.method = Method::Post;
                    self.url = "https://httpbin.org/post".to_string();
                    trigger_fetch = true;
                }
            });

            if trigger_fetch {
                let request = match self.method {
                    Method::Get => ureq::get(&self.url),
                    Method::Post => ureq::post(&self.url),
                };
                let method = self.method;
                let body = self.request_body.as_bytes().to_vec();
                let frame = frame.clone();
                let (sender, receiver) = std::sync::mpsc::channel();
                self.in_progress = Some(receiver);

                std::thread::spawn(move || {
                    let response = match method {
                        Method::Get => request.call(),
                        Method::Post => request.send_bytes(&body),
                    };

                    sender.send(response).ok();
                    frame.request_repaint();
                });
            }
        });
    }
}
