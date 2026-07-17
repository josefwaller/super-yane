use std::sync::{Arc, Mutex};

use eframe::CreationContext;
use egui::{Color32, ColorImage, Context, Frame, TextureHandle, Ui};
use egui_dock::{DockArea, DockState, NodeIndex};
use egui_infinite_scroll::InfiniteScroll;
use muda::Menu;
use super_yane::ppu::SCREEN_RESOLUTION;

use crate::{
    Console, Engine,
    emulation::Emulation,
    engine::Command,
    menu::spawn_menu_thread,
    ui::{EmuTab, TabViewer, cpu_data},
};

pub struct App {
    engine: Engine,
    screen_data: Option<TextureHandle>,
    tree: Arc<Mutex<DockState<(egui::Id, EmuTab)>>>,
    scroll: InfiniteScroll<i32, i32>,
    // Menu needs to be kept in scope
    menu: Menu,
}

impl App {
    pub fn new(cc: &CreationContext<'_>, console: Console, menu: Menu) -> Self {
        // Set up iniital layout
        let mut tree = DockState::new(vec![EmuTab::Screen]);
        let s = tree.main_surface_mut();
        let [main, _binary] = s.split_below(NodeIndex::root(), 0.75, vec![EmuTab::BinaryData]);
        let [main, _debug] = s.split_right(main, 0.85, vec![EmuTab::CpuDisassembly]);
        // 33 because the panel should now be one third the size of the container
        s.split_left(
            main,
            0.33,
            vec![EmuTab::Cpu, EmuTab::Ppu, EmuTab::DmaChannels],
        );
        let tree = Arc::new(Mutex::new(
            tree.map_tabs(|tab| (egui::Id::new(rand::random::<i32>()), *tab)),
        ));
        // Initialize emulation
        let emulation = Arc::new(Mutex::new(Emulation::new(console, cc.egui_ctx.clone())));
        // Initialize menu thread
        spawn_menu_thread(emulation.clone(), tree.clone());
        App {
            engine: Engine::new(emulation),
            screen_data: None,
            tree,
            scroll: InfiniteScroll::new().end_loader(|cursor, callback| {
                let start = cursor.unwrap_or(0);
                let end = 1;
                callback(Ok(((start..end).collect(), Some(end))));
            }),
            menu,
        }
    }
    fn initialize_texture(ui: &mut Ui) -> TextureHandle {
        ui.load_texture(
            "screen_data",
            ColorImage::filled(SCREEN_RESOLUTION, Color32::RED),
            Default::default(),
        )
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        // Get lock on console
        let emu_arc = self.engine.emulation.clone();

        // Update screen data
        let tex = self
            .screen_data
            .get_or_insert_with(|| App::initialize_texture(ui));
        {
            let emu = emu_arc.lock().expect("Unable to get a lock on emulation");
            tex.set(
                ColorImage::from_rgb(SCREEN_RESOLUTION, &emu.screen_data_rgb),
                Default::default(),
            );
        }
        // Gather nodes to add
        let mut added_nodes = vec![];
        let mut tree = self.tree.lock().unwrap();
        let mut style = egui_dock::Style::default();
        DockArea::new(&mut *tree)
            .show_add_buttons(true)
            .show_add_popup(true)
            .show_leaf_collapse_buttons(false)
            .show_inside(
                ui,
                &mut TabViewer {
                    engine: &mut self.engine,
                    screen: tex,
                    added_tabs: &mut added_nodes,
                },
            );
        for (path, tab) in added_nodes {
            tree.set_focused_node_and_surface(path);
            tree.push_to_focused_leaf((egui::Id::new(rand::random::<i32>()), tab));
        }
    }
}
