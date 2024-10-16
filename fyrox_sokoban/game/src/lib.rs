//! Game project.
use fyrox::{
    asset::untyped::ResourceKind,
    core::{algebra::Vector3, pool::Handle, reflect::prelude::*, visitor::prelude::*},
    event::Event,
    gui::message::UiMessage,
    material::{Material, MaterialResource},
    plugin::{Plugin, PluginContext, PluginRegistrationContext},
    scene::{
        base::BaseBuilder,
        camera::{CameraBuilder, OrthographicProjection, Projection, SkyBox},
        dim2::rectangle::RectangleBuilder,
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
    caisse: MaterialResource,
    caisse_ok: MaterialResource,
    mario_bas: MaterialResource,
    mario_droite: MaterialResource,
    mario_gauche: MaterialResource,
    mario_haut: MaterialResource,
    mur: MaterialResource,
    sol: MaterialResource,
    objectif: MaterialResource,
}

impl Images {
    fn load_material(context: &PluginContext, path: impl AsRef<Path>) -> MaterialResource {
        let texture_resource = context.resource_manager.request(path);

        let mut material = Material::standard_2d();
        material
            .set_texture(&"diffuseTexture".into(), Some(texture_resource))
            .unwrap();

        // TODO: Texture::set_magnification_filter(TextureMagnificationFilter::Nearest);
        // TODO: Texture::set_mignification_filter(TextureMagnificationFilter::Nearest);

        MaterialResource::new_ok(ResourceKind::Embedded, material)
    }

    pub fn load(context: &PluginContext) -> Self {
        Images {
            caisse: Self::load_material(context, "data/images/caisse.jpg"),
            caisse_ok: Self::load_material(context, "data/images/caisse_ok.jpg"),
            mario_bas: Self::load_material(context, "data/images/mario_bas.gif"),
            mario_droite: Self::load_material(context, "data/images/mario_droite.gif"),
            mario_gauche: Self::load_material(context, "data/images/mario_gauche.gif"),
            mario_haut: Self::load_material(context, "data/images/mario_haut.gif"),
            mur: Self::load_material(context, "data/images/mur.jpg"),
            sol: Default::default(),
            objectif: Self::load_material(context, "data/images/objectif.png"),
        }
    }
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

impl Game {
    fn draw_texture(scene: &mut Scene, material: MaterialResource, i: u32, j: u32) {
        let base_builder = BaseBuilder::new().with_local_transform(
            TransformBuilder::new()
                // Size of the rectangle is defined only by scale.
                .with_local_position(Vector3::new(i as f32, j as f32, 0.))
                .build(),
        );

        RectangleBuilder::new(base_builder)
            .with_material(material)
            .build(&mut scene.graph);
    }
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

        self.board = LoadingState::WaitingScene(board);
        self.images = Some(Images::load(&context));
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

        let (width, height) = (board.width(), board.height());

        // TODO: inverser la camera pour pouvoir utiliser des coordonnées positives ?
        CameraBuilder::new(
            BaseBuilder::new().with_local_transform(
                TransformBuilder::new()
                    .with_local_position(Vector3::new(
                        ((width as f32) - 1.) / 2.,
                        ((height as f32) - 1.) / 2.,
                        -5.,
                    ))
                    .build(),
            ),
        )
        .with_projection(Projection::Orthographic(OrthographicProjection {
            vertical_size: (height as f32) / 2.,
            ..Default::default()
        }))
        .with_skybox(SkyBox::default())
        .build(&mut scene.graph);

        let images = self.images.as_ref().expect("Images should be loaded.");

        for j in 0..height {
            for i in 0..width {
                use CellKind::*;
                match board.get(i, j) {
                    BoardElem(_, Void) => (),
                    BoardElem(_, Wall) => Self::draw_texture(scene, images.mur.clone(), i, j),
                    BoardElem(_, Floor) => Self::draw_texture(scene, images.sol.clone(), i, j),
                    BoardElem(_, Target) => {
                        // TODO: il serait mieux d'enlever la transparence avec la couleur du sol ?
                        Self::draw_texture(scene, images.sol.clone(), i, j);
                        Self::draw_texture(scene, images.objectif.clone(), i, j);
                    }
                }
            }
        }

        // TODO: register players and crates
    }
}
