use std::collections::BTreeMap;

use eframe::{egui, epi};

#[derive(Default)]
pub struct View {
    pub result: Option<Result<Resource, String>>,
    manager: TextureManager,
}

impl crate::Panel for View {
    fn name(&self) -> &'static str {
        "Center View Panel"
    }

    fn show(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let manager = &mut self.manager;
            if let Some(result) = &self.result {
                match result {
                    Ok(resource) => {
                        let Resource {
                            url,
                            status,
                            status_text,
                            bytes,
                            headers,
                            kind,
                        } = resource;

                        ui.monospace(format!("url:          {}", url));
                        ui.monospace(format!("status:       {} ({})", status, status_text));
                        ui.monospace(format!("content-type: {:?}", headers.get("content-type")));
                        ui.monospace(format!(
                            "size:         {:.1} kB",
                            bytes.len() as f32 / 1000.0
                        ));

                        ui.separator();

                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                egui::CollapsingHeader::new("Response headers")
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        egui::Grid::new("response_headers")
                                            .spacing(egui::vec2(
                                                ui.spacing().item_spacing.x * 2.0,
                                                0.0,
                                            ))
                                            .show(ui, |ui| {
                                                for header in headers {
                                                    ui.label(header.0);
                                                    ui.label(header.1);
                                                    ui.end_row();
                                                }
                                            })
                                    });

                                ui.separator();

                                match kind {
                                    Kind::Image(image) => {
                                        if let Some(texture_id) = manager.texture(frame, url, image)
                                        {
                                            let mut size = egui::Vec2::new(
                                                image.size[0] as f32,
                                                image.size[1] as f32,
                                            );
                                            size *= (ui.available_width() / size.x).min(1.0);
                                            ui.image(texture_id, size);
                                        }
                                    }
                                    Kind::Text(text) => {
                                        let tooltip = "Click to copy the response body";
                                        if ui.button("📋").on_hover_text(tooltip).clicked() {
                                            ui.output().copied_text = text.clone();
                                        }
                                        ui.separator();
                                        selectable_text(ui, text);
                                    }
                                    Kind::None => (),
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
        });
    }
}

pub struct Resource {
    /// The URL we ended up at. This can differ from the request url when we have followed redirects.
    pub url: String,
    /// Status code (e.g. `404` for "File not found").
    pub status: u16,
    /// Status text (e.g. "File not found" for status code `404`).
    pub status_text: String,
    /// The raw bytes.
    pub bytes: Vec<u8>,
    /// The returned headers. All header names are lower-case.
    pub headers: BTreeMap<String, String>,
    /// The resource kind (e.g. an image).
    kind: Kind,
}

impl Resource {
    pub fn from_response(
        _ctx: &egui::Context,
        response: Result<ureq::Response, ureq::Error>,
    ) -> Result<Self, String> {
        let response = match response {
            Ok(resp) => resp,
            Err(ureq::Error::Status(_, resp)) => resp, // Still read the body on e.g. 404
            Err(ureq::Error::Transport(error)) => return Err(error.to_string()),
        };

        let url = response.get_url().to_owned();
        let status = response.status();
        let status_text = response.status_text().to_owned();
        let mut headers = BTreeMap::new();
        for key in &response.headers_names() {
            if let Some(value) = response.header(key) {
                // lowercase for easy lookup
                headers.insert(key.to_ascii_lowercase(), value.to_owned());
            }
        }

        let mut reader = response.into_reader();
        let mut bytes = vec![];
        use std::io::Read;
        reader
            .read_to_end(&mut bytes)
            .map_err(|err| err.to_string())?;

        let kind = match headers.get("content-type") {
            Some(s) => {
                if s.starts_with("image/") {
                    match decode_image(&bytes) {
                        Some(i) => Kind::Image(i),
                        None => Kind::None,
                    }
                } else {
                    Kind::Text(String::from_utf8_lossy(&bytes).to_string())
                }
            }
            None => Kind::None,
        };

        // let element = scraper::Html::parse_document(response.text().unwrap());
        // println!("{:#?}", element);

        Ok(Self {
            url,
            status,
            status_text,
            bytes,
            headers,
            kind,
        })
    }
}

enum Kind {
    Image(epi::Image),
    Text(String),
    None,
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

fn selectable_text(ui: &mut egui::Ui, mut text: &str) {
    ui.add(
        egui::TextEdit::multiline(&mut text)
            .desired_width(f32::INFINITY)
            .text_style(egui::TextStyle::Monospace),
    );
}

fn decode_image(bytes: &[u8]) -> Option<epi::Image> {
    use image::GenericImageView;
    let image = image::load_from_memory(bytes).ok()?;
    let image_buffer = image.to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let pixels = image_buffer.into_vec();
    Some(epi::Image::from_rgba_unmultiplied(size, &pixels))
}
