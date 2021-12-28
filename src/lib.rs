mod bar;

use eframe::{egui, epi};

/// A widget specific to panel usage
pub trait Panel {
    /// `&'static` so we can also use it as a key to store state.
    fn name(&self) -> &'static str;

    /// Show the panel
    fn show(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame);
}

struct Resource {
    /// HTTP response
    response: ureq::Response,

    text: Option<String>,

    /// If set, the response was an image.
    image: Option<epi::Image>,
}

// impl Resource {
//     fn from_response(_ctx: &egui::Context, response: ureq::Response) -> Self {
//         let content_type = response.content_type();
//         let image = if content_type.starts_with("image/") {
//             decode_image(&response.bytes)
//         } else {
//             None
//         };
//
//         let text = response.text().map(|s| s.to_owned());
//         // let element = scraper::Html::parse_document(response.text().unwrap());
//         // println!("{:#?}", element);
//
//         Self {
//             response,
//             text,
//             image,
//         }
//     }
// }

#[derive(Default)]
pub struct WebApp {
    bar: bar::TopBar,

    result: Option<Result<Resource, ureq::Error>>,
    // manager: TextureManager,
}

impl epi::App for WebApp {
    fn name(&self) -> &str {
        "Web0"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        // if let Some(receiver) = &mut self.bar.in_progress {
        //     // Are we there yet?
        //     if let Ok(result) = receiver.try_recv() {
        //         self.bar.in_progress = None;
        //         self.result = Some(result.map(|response| Resource::from_response(ctx, response)));
        //     }
        // }

        self.bar.show(ctx, frame);

        // egui::CentralPanel::default().show(ctx, |ui| {
        //     if self.bar.in_progress.is_some() {
        //         ui.label("Loading…");
        //     } else {
        //         self.ui_resource(ui, frame);
        //     }
        // });
    }
}

// impl WebApp {
//     fn ui_resource(&mut self, ui: &mut egui::Ui, frame: &epi::Frame) {
//         let manager = &mut self.manager;
//         if let Some(result) = &self.result {
//             match result {
//                 Ok(resource) => {
//                     let Resource {
//                         response,
//                         text,
//                         image,
//                     } = resource;
//
//                     ui.monospace(format!("url:          {}", response.get_url()));
//                     ui.monospace(format!(
//                         "status:       {} ({})",
//                         response.status(),
//                         response.status_text()
//                     ));
//                     ui.monospace(format!("content-type: {}", response.content_type()));
//                     ui.monospace(format!(
//                         "size:         {:.1} kB",
//                         response.bytes.len() as f32 / 1000.0
//                     ));
//
//                     ui.separator();
//
//                     egui::ScrollArea::vertical()
//                         .auto_shrink([false; 2])
//                         .show(ui, |ui| {
//                             egui::CollapsingHeader::new("Response headers")
//                                 .default_open(false)
//                                 .show(ui, |ui| {
//                                     egui::Grid::new("response_headers")
//                                         .spacing(egui::vec2(ui.spacing().item_spacing.x * 2.0, 0.0))
//                                         .show(ui, |ui| {
//                                             for header in &response.headers {
//                                                 ui.label(header.0);
//                                                 ui.label(header.1);
//                                                 ui.end_row();
//                                             }
//                                         })
//                                 });
//
//                             ui.separator();
//
//                             if let Some(text) = &text {
//                                 let tooltip = "Click to copy the response body";
//                                 if ui.button("📋").on_hover_text(tooltip).clicked() {
//                                     ui.output().copied_text = text.clone();
//                                 }
//                                 ui.separator();
//                             }
//
//                             if let Some(image) = image {
//                                 if let Some(texture_id) =
//                                     manager.texture(frame, &response.url, image)
//                                 {
//                                     let mut size =
//                                         egui::Vec2::new(image.size[0] as f32, image.size[1] as f32);
//                                     size *= (ui.available_width() / size.x).min(1.0);
//                                     ui.image(texture_id, size);
//                                 }
//                             } else if let Some(text) = &text {
//                                 selectable_text(ui, text);
//                             } else {
//                                 // We don't care lossy text.
//                                 selectable_text(ui, unsafe {
//                                     std::str::from_utf8_unchecked(&response.bytes)
//                                 });
//                             }
//                         });
//                 }
//                 Err(error) => {
//                     // This should only happen if the fetch API isn't available or something similar.
//                     ui.colored_label(
//                         egui::Color32::RED,
//                         if error.is_empty() { "Error" } else { error },
//                     );
//                 }
//             }
//         }
//     }
// }
//
// fn selectable_text(ui: &mut egui::Ui, mut text: &str) {
//     ui.add(
//         egui::TextEdit::multiline(&mut text)
//             .desired_width(f32::INFINITY)
//             .text_style(egui::TextStyle::Monospace),
//     );
// }
//
// // ----------------------------------------------------------------------------
// // Texture/image handling is very manual at the moment.
//
// /// Immediate mode texture manager that supports at most one texture at the time :)
// #[derive(Default)]
// struct TextureManager {
//     loaded_url: String,
//     texture_id: Option<egui::TextureId>,
// }
//
// impl TextureManager {
//     fn texture(
//         &mut self,
//         frame: &epi::Frame,
//         url: &str,
//         image: &epi::Image,
//     ) -> Option<egui::TextureId> {
//         if self.loaded_url != url {
//             if let Some(texture_id) = self.texture_id.take() {
//                 frame.free_texture(texture_id);
//             }
//
//             self.texture_id = Some(frame.alloc_texture(image.clone()));
//             self.loaded_url = url.to_owned();
//         }
//         self.texture_id
//     }
// }
//
// fn decode_image(bytes: &[u8]) -> Option<epi::Image> {
//     use image::GenericImageView;
//     let image = image::load_from_memory(bytes).ok()?;
//     let image_buffer = image.to_rgba8();
//     let size = [image.width() as usize, image.height() as usize];
//     let pixels = image_buffer.into_vec();
//     Some(epi::Image::from_rgba_unmultiplied(size, &pixels))
// }
