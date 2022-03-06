use database::Database;
use eframe::{
    epi::{App, Frame},
    NativeOptions,
};
use egui::{Context, Visuals};
use example::Example;
use traits::Container;

mod example;
mod traits;

struct KryptaApp {
    database: Database,
    example: Example,
}

impl KryptaApp {
    fn new(database: Database) -> Self {
        KryptaApp {
            database,
            example: Example::default(),
        }
    }
}

impl App for KryptaApp {
    fn name(&self) -> &str {
        "Krypta"
    }

    fn update(&mut self, ctx: &Context, _frame: &Frame) {
        ctx.set_visuals(Visuals::dark());

        self.example.show(ctx, &mut true);
    }
}

pub fn show_gui(database: Database) {
    let app = KryptaApp::new(database);
    eframe::run_native(Box::new(app), NativeOptions::default());
}
