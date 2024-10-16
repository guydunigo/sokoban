//! Game project.
use fyrox::{
    asset::{untyped::ResourceKind, Resource},
    core::{
        algebra::Vector3, color::Color, pool::Handle, reflect::prelude::*, visitor::prelude::*,
    },
    event::Event,
    gui::message::UiMessage,
    material::{Material, MaterialResource},
    plugin::{Plugin, PluginContext, PluginRegistrationContext},
    resource::texture::Texture,
    scene::{
        base::BaseBuilder,
        camera::{CameraBuilder, OrthographicProjection, Projection},
        dim2::rectangle::{Rectangle, RectangleBuilder},
        node::Node,
        transform::TransformBuilder,
        Scene,
    },
};
use sokoban::{Board, BoardElem, CellKind, Direction};
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
pub enum LoadingState {
    #[default]
    None,
    WaitingScene(Board),
    SceneFilled {
        board: Board,
        scene: Handle<Scene>,
    },
}

#[derive(Default, Visit, Reflect, Debug)]
pub struct Game {
    images: Option<Images>,
    board: LoadingState,
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

        let images = Images {
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
        };

        self.board = LoadingState::WaitingScene(board);
        self.images = Some(images);
    }

    fn on_deinit(&mut self, _context: PluginContext) {
        // Do a cleanup here.
    }

    fn update(&mut self, _context: &mut PluginContext) {
        // Add your global update code here.
        // self.board.unwrap().player
    }

    fn on_os_event(&mut self, _event: &Event<()>, _context: PluginContext) {
        // Do something on OS event here.
    }

    fn on_ui_message(&mut self, _context: &mut PluginContext, _message: &UiMessage) {
        // Handle UI events here.
    }

    fn on_scene_begin_loading(&mut self, _path: &Path, ctx: &mut PluginContext) {
        if let LoadingState::SceneFilled { scene, .. } = self.board {
            if scene.is_some() {
                ctx.scenes.remove(scene);
            }
        }
    }

    fn on_scene_loaded(
        &mut self,
        _path: &Path,
        scene: Handle<Scene>,
        _data: &[u8],
        context: &mut PluginContext,
    ) {
        let scene = context.scenes.try_get_mut(scene).unwrap();

        let LoadingState::WaitingScene(ref board) = self.board else {
            panic!("Should be in loading state WaitingScene with a loaded board !");
        };

        let (width, height) = (board.width() as f32, board.height() as f32);

        // TODO: change projection params (see defaults) ?
        // https://docs.rs/fyrox-impl/0.34.1/src/fyrox_impl/scene/camera.rs.html#112
        CameraBuilder::new(
            BaseBuilder::new().with_local_transform(
                TransformBuilder::new()
                    .with_local_position(Vector3::new((width - 1.) / 2., (height - 1.) / 2., -5.))
                    .build(),
            ),
        )
        .with_projection(Projection::Orthographic(OrthographicProjection {
            vertical_size: height / 2.,
            ..Default::default()
        }))
        .build(&mut scene.graph);

        let images = self.images.as_ref().expect("Images should be loaded.");

        // TODO: direct dans image
        // TODO: pixel art
        let mut material = Material::standard_2d();
        material
            .set_texture(&"diffuseTexture".into(), Some(images.caisse.clone()))
            .unwrap();
        let material_resource = MaterialResource::new_ok(ResourceKind::Embedded, material);

        for j in 0..board.height() {
            for i in 0..board.width() {
                let base_builder = BaseBuilder::new().with_local_transform(
                    TransformBuilder::new()
                        // Size of the rectangle is defined only by scale.
                        .with_local_position(Vector3::new(i as f32, j as f32, 0.))
                        .build(),
                );
                RectangleBuilder::new(base_builder)
                    .with_color(Color::WHITE)
                    .with_material(material_resource.clone())
                    .build(&mut scene.graph);

                /*
                use CellKind::*;
                match board.get(i, j) {
                    BoardElem(_, Void) => (),
                    BoardElem(_, Wall) => canvas.draw(&images.mur, params),
                    BoardElem(_, Floor) => canvas.draw(&rect, params),
                    BoardElem(_, Target) => {
                        canvas.draw(&rect, params);
                        canvas.draw(&self.images.objectif, params);
                    }
                }
                */
            }
        }
    }
}
