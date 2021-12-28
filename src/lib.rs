use eframe::{egui, epi};
use std::sync::mpsc::Receiver;

struct Resource {
    /// HTTP response
    response: ehttp::Response,

    text: Option<String>,

    /// If set, the response was an image.
    image: Option<epi::Image>,
}

impl Resource {
    fn from_response(_ctx: &egui::Context, response: ehttp::Response) -> Self {
        let content_type = response.content_type().unwrap_or_default();
        let image = if content_type.starts_with("image/") {
            decode_image(&response.bytes)
        } else {
            None
        };

        let text = response.text().map(|s| s.to_owned());

        Self {
            response,
            text,
            image,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
enum Method {
    Get,
    Post,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct WebApp {
    url: String,

    method: Method,

    request_body: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    in_progress: Option<Receiver<Result<ehttp::Response, String>>>,

    #[cfg_attr(feature = "serde", serde(skip))]
    result: Option<Result<Resource, String>>,

    #[cfg_attr(feature = "serde", serde(skip))]
    tex_mngr: TextureManager,
}

impl Default for WebApp {
    fn default() -> Self {
        Self {
            url: "https://raw.githubusercontent.com/emilk/egui/master/README.md".to_owned(),
            method: Method::Get,
            request_body: Default::default(),
            in_progress: Default::default(),
            result: Default::default(),
            tex_mngr: Default::default(),
        }
    }
}

impl epi::App for WebApp {
    fn name(&self) -> &str {
        "Web0"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        if let Some(receiver) = &mut self.in_progress {
            // Are we there yet?
            if let Ok(result) = receiver.try_recv() {
                self.in_progress = None;
                self.result = Some(result.map(|response| Resource::from_response(ctx, response)));
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let trigger_fetch = self.ui_url(ui);

            if trigger_fetch {
                let request = match self.method {
                    Method::Get => ehttp::Request::get(&self.url),
                    Method::Post => {
                        ehttp::Request::post(&self.url, self.request_body.as_bytes().to_vec())
                    }
                };
                let frame = frame.clone();
                let (sender, receiver) = std::sync::mpsc::channel();
                self.in_progress = Some(receiver);

                ehttp::fetch(request, move |response| {
                    sender.send(response).ok();
                    frame.request_repaint();
                });
            }

            ui.separator();

            if self.in_progress.is_some() {
                ui.label("Loading…");
            } else {
                self.ui_resource(ui, frame);
            }
        });
    }
}

impl WebApp {
    fn ui_url(&mut self, ui: &mut egui::Ui) -> bool {
        let url = &mut self.url;
        let mut trigger_fetch = false;

        ui.horizontal(|ui| {
            ui.label("URL:");
            trigger_fetch |= ui
                .add(egui::TextEdit::singleline(url).desired_width(f32::INFINITY))
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
                *url = format!("https://picsum.photos/seed/{}/{}", seed, side);
                trigger_fetch = true;
            }

            if ui.button("POST to httpbin.org").clicked() {
                self.method = Method::Post;
                *url = format!("https://httpbin.org/post");
                trigger_fetch = true;
            }
        });

        trigger_fetch
    }

    fn ui_resource(&mut self, ui: &mut egui::Ui, frame: &epi::Frame) {
        let tex_mngr = &mut self.tex_mngr;
        if let Some(result) = &self.result {
            match result {
                Ok(resource) => {
                    let Resource {
                        response,
                        text,
                        image,
                    } = resource;

                    ui.monospace(format!("url:          {}", response.url));
                    ui.monospace(format!(
                        "status:       {} ({})",
                        response.status, response.status_text
                    ));
                    ui.monospace(format!(
                        "content-type: {}",
                        response.content_type().unwrap_or_default()
                    ));
                    ui.monospace(format!(
                        "size:         {:.1} kB",
                        response.bytes.len() as f32 / 1000.0
                    ));

                    ui.separator();

                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            egui::CollapsingHeader::new("Response headers")
                                .default_open(false)
                                .show(ui, |ui| {
                                    egui::Grid::new("response_headers")
                                        .spacing(egui::vec2(ui.spacing().item_spacing.x * 2.0, 0.0))
                                        .show(ui, |ui| {
                                            for header in &response.headers {
                                                ui.label(header.0);
                                                ui.label(header.1);
                                                ui.end_row();
                                            }
                                        })
                                });

                            ui.separator();

                            if let Some(text) = &text {
                                let tooltip = "Click to copy the response body";
                                if ui.button("📋").on_hover_text(tooltip).clicked() {
                                    ui.output().copied_text = text.clone();
                                }
                                ui.separator();
                            }

                            if let Some(image) = image {
                                if let Some(texture_id) =
                                    tex_mngr.texture(frame, &response.url, image)
                                {
                                    let mut size =
                                        egui::Vec2::new(image.size[0] as f32, image.size[1] as f32);
                                    size *= (ui.available_width() / size.x).min(1.0);
                                    ui.image(texture_id, size);
                                }
                            } else if let Some(text) = &text {
                                selectable_text(ui, text);
                            } else {
                                // We don't care lossy text.
                                selectable_text(ui, unsafe {
                                    std::str::from_utf8_unchecked(&response.bytes)
                                });
                            }
                        });
                }
                Err(error) => {
                    // This should only happen if the fetch API isn't available or something similar.
                    ui.colored_label(
                        egui::Color32::RED,
                        if error.is_empty() { "Error" } else { error },
                    );
                }
            }
        }
    }
}

fn selectable_text(ui: &mut egui::Ui, mut text: &str) {
    ui.add(
        egui::TextEdit::multiline(&mut text)
            .desired_width(f32::INFINITY)
            .text_style(egui::TextStyle::Monospace),
    );
}

// ----------------------------------------------------------------------------
// Texture/image handling is very manual at the moment.

/// Immediate mode texture manager that supports at most one texture at the time :)
#[derive(Default)]
struct TextureManager {
    loaded_url: String,
    texture_id: Option<egui::TextureId>,
}

impl TextureManager {
    fn texture(
        &mut self,
        frame: &epi::Frame,
        url: &str,
        image: &epi::Image,
    ) -> Option<egui::TextureId> {
        if self.loaded_url != url {
            if let Some(texture_id) = self.texture_id.take() {
                frame.free_texture(texture_id);
            }

            self.texture_id = Some(frame.alloc_texture(image.clone()));
            self.loaded_url = url.to_owned();
        }
        self.texture_id
    }
}

fn decode_image(bytes: &[u8]) -> Option<epi::Image> {
    use image::GenericImageView;
    let image = image::load_from_memory(bytes).ok()?;
    let image_buffer = image.to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let pixels = image_buffer.into_vec();
    Some(epi::Image::from_rgba_unmultiplied(size, &pixels))
}
