use egui::TextureHandle;
use egui_dock::NodePath;
use strum::IntoEnumIterator;

use crate::{
    app::AppState,
    engine::Engine,
    ui::emu_tab::{EmuTab, render_tab_pane},
};

pub struct TabViewer<'a> {
    pub engine: &'a mut Engine,
    pub added_tabs: &'a mut Vec<(NodePath, EmuTab)>,
    pub app_state: &'a mut AppState,
}

impl<'a> egui_dock::TabViewer for TabViewer<'a> {
    type Tab = (egui::Id, EmuTab);
    fn title(&mut self, (_, tab): &mut Self::Tab) -> egui::WidgetText {
        egui::WidgetText::Text(tab.to_string())
    }
    fn id(&mut self, (id, _): &mut Self::Tab) -> egui::Id {
        *id
    }
    fn ui(&mut self, ui: &mut egui::Ui, (_, tab): &mut Self::Tab) {
        // Move command out to respect rust memory safety rules
        {
            let mut emu = self
                .engine
                .emulation
                .lock()
                .expect("Unable to get a lock on Emulation");
            render_tab_pane(ui, *tab, &mut emu, self.app_state);
        }
        // if let Some(command) = to_send {
        //     self.engine.update(command);
        // }
    }
    fn add_popup(&mut self, ui: &mut egui::Ui, path: egui_dock::NodePath) {
        ui.vertical(|ui| {
            EmuTab::iter().for_each(|tab| {
                if ui.selectable_label(false, tab.to_string()).clicked() {
                    self.added_tabs.push((path, tab));
                }
            });
        });
    }
}
