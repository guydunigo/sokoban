//! Game project.
use fyrox::{
    asset::Resource,
    core::{pool::Handle, reflect::prelude::*, visitor::prelude::*},
    event::Event,
    gui::message::UiMessage,
    plugin::{Plugin, PluginContext, PluginRegistrationContext},
    resource::texture::Texture,
    scene::{dim2::rectangle::Rectangle, node::Node, Scene},
};
use sokoban::{Board, Direction};
use std::{env::args, fs::read_to_string, path::Path, str::FromStr};

const DEFAULT_LEVEL_FILENAME: &str = "../map.txt";

// Re-export the engine.
pub use fyrox;

#[derive(Default, Visit, Reflect, Debug)]
struct Images {
    caisse: Resource<Texture>,
    caisse_ok: Resource<Texture>,
    mario_bas: Resource<Texture>,
    mario_droite: Resource<Texture>,
    mario_gauche: Resource<Texture>,
    mario_haut: Resource<Texture>,
    mur: Resource<Texture>,
    objectif: Resource<Texture>,
}

#[derive(Default, Visit, Reflect, Debug)]
pub struct Game {
    scene: Handle<Scene>,
    images: Option<Images>,
    board: Option<Board>,
    direction: Direction,
}

impl Plugin for Game {
    fn register(&self, _context: PluginRegistrationContext) {
        // Register your scripts here.
    }

    fn init(&mut self, scene_path: Option<&str>, context: PluginContext) {
        context
            .async_scene_loader
            .request(scene_path.unwrap_or("data/scene.rgs"));

        // TODO: better error handling
        let board = {
            let arg1 = args().nth(1);
            let level_filename = arg1.as_ref().map_or(DEFAULT_LEVEL_FILENAME, |f| &f[..]);

            let level = read_to_string(level_filename)
                .expect(&format!("Could not open file `{level_filename}`.")[..]);

            Board::from_str(&level[..]).expect("Failed to load level !")
        };

        self.board = Some(board);
        self.images = Some(Images {
            caisse: context.resource_manager.request("data/images/caisse.jpg"),
            caisse_ok: context
                .resource_manager
                .request("data/images/caisse_ok.jpg"),
            mario_bas: context
                .resource_manager
                .request("data/images/mario_bas.gif"),
            mario_droite: context
                .resource_manager
                .request("data/images/mario_droite.gif"),
            mario_gauche: context
                .resource_manager
                .request("data/images/mario_gauche.gif"),
            mario_haut: context
                .resource_manager
                .request("data/images/mario_haut.gif"),
            mur: context.resource_manager.request("data/images/mur.jpg"),
            objectif: context.resource_manager.request("data/images/objectif.png"),
        });
    }

    fn on_deinit(&mut self, _context: PluginContext) {
        // Do a cleanup here.
    }

    fn update(&mut self, _context: &mut PluginContext) {
        // Add your global update code here.
    }

    fn on_os_event(&mut self, _event: &Event<()>, _context: PluginContext) {
        // Do something on OS event here.
    }

    fn on_ui_message(&mut self, _context: &mut PluginContext, _message: &UiMessage) {
        // Handle UI events here.
    }

    fn on_scene_begin_loading(&mut self, _path: &Path, ctx: &mut PluginContext) {
        if self.scene.is_some() {
            ctx.scenes.remove(self.scene);
        }
    }

    fn on_scene_loaded(
        &mut self,
        _path: &Path,
        scene: Handle<Scene>,
        _data: &[u8],
        _context: &mut PluginContext,
    ) {
        self.scene = scene;
    }
}
