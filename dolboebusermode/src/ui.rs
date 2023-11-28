use imgui::Condition;
use obfstr::obfstr;

use crate::Application;

pub struct SettingsUI {}

impl SettingsUI {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&mut self, app: &Application, ui: &imgui::Ui) {
        ui.window(obfstr!("JustVAC"))
            .size([600.0, 300.0], Condition::FirstUseEver)
            .title_bar(false)
            .build(|| {
                {
                    ui.text_colored([0.86, 0.52, 0.24, 1.0], obfstr!("JustVAC"));

                    ui.dummy([0.0, 1.0]);
                }

                if let Some(_tab_bar) = ui.tab_bar("main") {
                    if let Some(_tab) = ui.tab_item(obfstr!("Визуалы")) {}
                }
            });
    }
}