use crate::gameoflife::Universe;

use std::sync::mpsc::{Receiver, Sender};

pub enum Pattern {
    Spaceship,
}

pub enum MessageState {
    Pattern(Pattern),
    Start,
    Pause,
    Clear,
}

pub struct App {
    sender: Sender<MessageState>,
    receiver: Receiver<String>,
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = cc.egui_ctx.clone();

        let (out_sender, out_receiver) = std::sync::mpsc::channel();
        let (in_sender, in_receiver) = std::sync::mpsc::channel::<MessageState>();

        let mut universe = Universe::default();
        std::thread::spawn(move || loop {
            universe.update(&ctx, &out_sender, &in_receiver);
            std::thread::sleep(std::time::Duration::from_millis(universe.interval.into()))
        });

        Self {
            sender: in_sender,
            receiver: out_receiver,
        }
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        // Receive the value from the sending thread.
        let value = self.receiver.recv().unwrap_or_else(|_| "".to_string());

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Settings");

            if ui.button("Start").clicked() {
                self.sender
                    .send(MessageState::Start)
                    .expect("Error while sending");
            }

            if ui.button("Stop").clicked() {
                self.sender
                    .send(MessageState::Pause)
                    .expect("Error while sending");
            }

            if ui.button("Clear").clicked() {
                self.sender
                    .send(MessageState::Clear)
                    .expect("Error while sending");
            }

            if ui.button("Gliders").clicked() {
                self.sender
                    .send(MessageState::Pattern(Pattern::Spaceship))
                    .expect("Error while sending");
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Conway's Game of Life");
            ui.label(value);
        });
    }
}
