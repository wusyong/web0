mod center;
mod top;

use center::Resource;
use eframe::{egui, epi};

/// A widget specific to panel usage
pub trait Panel {
    /// `&'static` so we can also use it as a key to store state.
    fn name(&self) -> &'static str;

    /// Show the panel
    fn show(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame);
}

#[derive(Default)]
pub struct WebApp {
    bar: top::Bar,
    view: center::View,
}

impl epi::App for WebApp {
    fn name(&self) -> &str {
        "Web0"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        if let Some(receiver) = &mut self.bar.in_progress {
            // Are we there yet?
            if let Ok(result) = receiver.try_recv() {
                self.bar.in_progress = None;
                self.view.result = Some(Resource::from_response(ctx, result));
            }
        }

        self.bar.show(ctx, frame);

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.bar.in_progress.is_some() {
                ui.label("Loading…");
            } else {
                self.view.show(ctx, frame);
            }
        });
    }
}
