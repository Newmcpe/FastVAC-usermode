use std::{
    cell::RefCell,
    error::Error,
    fmt::Debug,
    rc::Rc,
};
use std::io::Write;
use std::sync::Arc;

use anyhow::Context;
use imgui::{Condition, FontConfig, FontGlyphRanges, FontId, FontSource, Key};
use obfstr::obfstr;
use windows::Win32::System::Console::GetConsoleProcessList;

use overlay::{LoadingError, OverlayError, OverlayOptions, OverlayTarget};
use shared::kernel_request::Driver;

use crate::cs_interface::CSInterface;
use crate::ui::SettingsUI;

mod ui;
mod cs_interface;


#[allow(non_snake_case)]
#[tokio::main]
async fn main() {
    let driver = Arc::new(Driver::init());

    let driver = driver.clone();
    let dll_base = driver.get_clientdll_base();

    println!("dll_base: 0x{:X}", dll_base);
}


pub struct AppFonts {
    default: FontId,
}

fn is_console_invoked() -> bool {
    let console_count = unsafe {
        let mut result = [0u32; 128];
        GetConsoleProcessList(&mut result)
    };

    console_count > 1
}

fn show_critical_error(message: &str) {
    for line in message.lines() {
        log::error!("{}", line);
    }

    if !is_console_invoked() {
        overlay::show_error_message(obfstr!("FastVACation"), message);
    }
}

pub struct Application {
    kernel_interface: CSInterface,
    fonts: AppFonts,
    settings_ui: RefCell<SettingsUI>,
    settings_visible: bool,
}

impl Application {
    pub fn update(&mut self, ui: &imgui::Ui) -> anyhow::Result<()> {
        if ui.is_key_pressed_no_repeat(Key::Insert) {
            log::info!("pause key pressed");
            self.settings_visible = !self.settings_visible;
        }

        Ok(())
    }

    fn render(&self, ui: &imgui::Ui) {
        ui.window("overlay")
            .draw_background(false)
            .no_decoration()
            .no_inputs()
            .size(ui.io().display_size, Condition::Always)
            .position([0.0, 0.0], Condition::Always)
            .build(|| self.render_overlay(ui));

        if self.settings_visible {
            let mut settings_ui = self.settings_ui.borrow_mut();
            settings_ui.render(self, ui)
        }
    }

    fn render_overlay(&self, ui: &imgui::Ui) {
        let text = format!("{:.2} FPS", ui.io().framerate);
        ui.set_cursor_pos([
            ui.window_size()[0] - ui.calc_text_size(&text)[0] - 10.0,
            6.0,
        ]);
        ui.text(text);
        let text = format!("CS2 Process ID: {}", self.kernel_interface.get_process_id());
        ui.set_cursor_pos([
            ui.window_size()[0] - ui.calc_text_size(&text)[0] - 10.0,
            18.0,
        ]);
        ui.text(text);
        let text = format!("client.dll base: 0x{:X}", self.kernel_interface.get_client_address());
        ui.set_cursor_pos([
            ui.window_size()[0] - ui.calc_text_size(&text)[0] - 10.0,
            30.0,
        ]);
        ui.text(text);
    }
}

fn overlay_main() -> anyhow::Result<()> {
    let kernel_interface = CSInterface::init();
    let app_fonts: Rc<RefCell<Option<AppFonts>>> = Default::default();
    let overlay_options = OverlayOptions {
        title: obfstr!("CS2 Overlay").to_string(),
        target: OverlayTarget::WindowOfProcess(kernel_interface.get_process_id()),
        font_init: Some(Box::new({
            let app_fonts = app_fonts.clone();

            move |imgui| {
                let mut app_fonts = app_fonts.borrow_mut();

                let font_size = 18.0;
                let default_font = imgui.fonts().add_font(&[FontSource::TtfData {
                    data: include_bytes!("../resources/Montserrat-Regular.ttf"),
                    size_pixels: font_size,
                    config: Some(FontConfig {
                        rasterizer_multiply: 1.5,
                        oversample_h: 4,
                        oversample_v: 4,
                        glyph_ranges: FontGlyphRanges::cyrillic(),
                        ..FontConfig::default()
                    }),
                }]);

                *app_fonts = Some(AppFonts {
                    default: default_font,
                });
            }
        })),
    };
    let mut overlay = match overlay::init(&overlay_options) {
        Err(OverlayError::VulkanDllNotFound(LoadingError::LibraryLoadFailure(source))) => {
            match &source {
                libloading::Error::LoadLibraryExW { .. } => {
                    let error = source.source().context("LoadLibraryExW to have a source")?;
                    let message = format!("Failed to load vulkan-1.dll.\nError: {:#}", error);
                    show_critical_error(&message);
                }
                error => {
                    let message = format!(
                        "An error occurred while loading vulkan-1.dll.\nError: {:#}",
                        error
                    );
                    show_critical_error(&message);
                }
            }
            return Ok(());
        }
        value => value?,
    };

    let application = Application {
        kernel_interface,
        settings_ui: RefCell::new(SettingsUI {}),
        settings_visible: false,
        fonts: app_fonts
            .borrow_mut()
            .take()
            .context("failed to initialize app fonts")?,
    };

    let application = Rc::new(RefCell::new(application));

    overlay.main_loop(
        {
            move |controller| {
                true
            }
        },
        move |ui| {
            let mut application = application.borrow_mut();

            if let Err(error) = application.update(ui) {
                log::error!("An error occurred while updating the overlay.\nError: {:#}", error);
            }

            application.render(ui);

            true
        },
    )
}