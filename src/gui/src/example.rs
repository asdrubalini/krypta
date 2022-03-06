use egui::{Context, Ui, Window};

use crate::traits::{Container, View};

#[derive(Default)]
pub struct Example {
    ciao: bool,
}

impl Container for Example {
    fn name(&self) -> &'static str {
        "Example"
    }

    fn show(&mut self, ctx: &Context, open: &mut bool) {
        Window::new(self.name())
            .open(open)
            .resizable(false)
            .show(ctx, |ui| {
                self.ui(ui);
            });
    }
}

impl View for Example {
    fn ui(&mut self, ui: &mut Ui) {
        if ui.button("cliccami").clicked() {
            self.ciao = !self.ciao;
        }

        if self.ciao {
            ui.label("Ciao");
        }
    }
}
