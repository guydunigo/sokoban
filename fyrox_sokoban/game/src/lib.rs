//! Game project.
use fyrox::{
    asset::{untyped::ResourceKind, Resource},
    core::{
        algebra::{UnitQuaternion, Vector3},
        pool::Handle,
        reflect::prelude::*,
        visitor::prelude::*,
    },
    event::{ElementState, Event, WindowEvent},
    gui::message::UiMessage,
    keyboard::{Key, NamedKey},
    material::{Material, MaterialResource},
    plugin::{Plugin, PluginContext, PluginRegistrationContext},
    resource::texture::{Texture, TextureMagnificationFilter, TextureMinificationFilter},
    scene::{
        base::BaseBuilder,
        camera::{CameraBuilder, OrthographicProjection, Projection, SkyBox},
        dim2::rectangle::RectangleBuilder,
        graph::Graph,
        node::Node,
        transform::TransformBuilder,
        Scene,
    },
};
use sokoban::{Board, BoardElem, CellKind, Direction, MovableItem};
use std::{env::args, fs::read_to_string, mem, path::Path, str::FromStr};

const DEFAULT_LEVEL_FILENAME: &str = "../map.txt";

// Re-export the engine.
pub use fyrox;

fn rotation() -> UnitQuaternion<f32> {
    UnitQuaternion::from_euler_angles(0., 0., std::f32::consts::PI)
}

fn material_for_player(images: &Images, direction: Direction) -> MaterialResource {
    use Direction::*;
    match direction {
        Up => images.mario_haut.clone(),
        Down => images.mario_bas.clone(),
        Left => images.mario_gauche.clone(),
        Right => images.mario_droite.clone(),
    }
}

fn material_for_crate(images: &Images, under: CellKind) -> MaterialResource {
    if under == CellKind::Target {
        images.caisse_ok.clone()
    } else {
        images.caisse.clone()
    }
}

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
    fn load_material(context: &mut PluginContext, path: impl AsRef<Path>) -> MaterialResource {
        let pathbuf = path.as_ref().to_path_buf();

        let texture_resource = context.resource_manager.request(path);

        let mut material = Material::standard_2d();
        material
            .set_texture(&"diffuseTexture".into(), Some(texture_resource))
            .unwrap();

        // TODO: So ugly...
        let resource_manager = context.resource_manager.clone();
        context.task_pool.spawn_plugin_task(
            async move {
                let texture_resource: Resource<Texture> =
                    resource_manager.request(pathbuf).await.unwrap();

                let mut texture = texture_resource.data_ref();
                texture.set_magnification_filter(TextureMagnificationFilter::Nearest);
                texture.set_minification_filter(TextureMinificationFilter::Nearest);
            },
            |_, _: &mut Game, _| {},
        );

        MaterialResource::new_ok(ResourceKind::Embedded, material)
    }

    pub fn load(context: &mut PluginContext) -> Self {
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
        player: Handle<Node>,
        crates: Vec<Handle<Node>>,
    },
}

impl LoadingState {
    fn unwrap_scene_filled(
        &mut self,
    ) -> (&mut Board, &Handle<Scene>, &Handle<Node>, &[Handle<Node>]) {
        if let LoadingState::SceneFilled {
            board,
            scene,
            player,
            crates,
        } = self
        {
            (board, scene, player, &crates[..])
        } else {
            panic!("Game should be in LoadingStata::SceneFilled with all the board loaded into the scene !");
        }
    }
}

#[derive(Default, Visit, Reflect, Debug)]
pub struct Game {
    images: Option<Images>,
    board: LoadingState,
    direction: Direction,
}

impl Game {
    fn create_rectangle(
        scene: &mut Scene,
        material: MaterialResource,
        i: u32,
        j: u32,
        z: f32,
    ) -> Handle<Node> {
        let base_builder = BaseBuilder::new().with_local_transform(
            TransformBuilder::new()
                // Size of the rectangle is defined only by scale.
                .with_local_position(Vector3::new(i as f32, j as f32, z))
                .with_local_rotation(rotation())
                .build(),
        );

        RectangleBuilder::new(base_builder)
            .with_material(material)
            .build(&mut scene.graph)
    }

    fn reset(&mut self, context: &mut PluginContext) {
        let (board, scene, player, crates) = self.board.unwrap_scene_filled();
        board.reset();
        self.direction = Direction::Down;
        let mut graph = &mut context.scenes.try_get_mut(*scene).unwrap().graph;

        Self::update_node_pos(&mut graph, *player, board.player());

        crates
            .iter()
            .zip(board.crates())
            .for_each(|(h, c)| Self::update_node_pos(graph, *h, c.pos()));
    }

    fn update_node_pos(graph: &mut Graph, node: Handle<Node>, (i, j): (u32, u32)) {
        let current_transform = graph[node].local_transform_mut();
        current_transform.set_position(Vector3::new(
            i as f32,
            j as f32,
            current_transform.position().z,
        ));
    }

    fn do_move_player(&mut self, context: &mut PluginContext, dir: Direction) {
        let (board, scene, player, _) = self.board.unwrap_scene_filled();
        if let Some(moved) = board.do_move_player(dir) {
            let graph = &mut context.scenes.try_get_mut(*scene).unwrap().graph;
            Self::update_node_pos(graph, *player, board.player());
            if let Some(moved) = moved {
                Self::update_node_pos(graph, *moved_crate, moved);
            }
        }
        self.direction = dir;
    }
}

impl Plugin for Game {
    fn register(&self, _context: PluginRegistrationContext) {
        // Register your scripts here.
    }

    fn init(&mut self, scene_path: Option<&str>, mut context: PluginContext) {
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
        self.images = Some(Images::load(&mut context));
    }

    fn on_deinit(&mut self, _context: PluginContext) {
        // Do a cleanup here.
    }

    fn update(&mut self, _context: &mut PluginContext) {
        // Add your global update code here.
        // self.board.unwrap().player
    }

    fn on_os_event(&mut self, event: &Event<()>, mut context: PluginContext) {
        // Do something on OS event here.
        if let Event::WindowEvent { event, .. } = event {
            if let WindowEvent::KeyboardInput { event, .. } = event {
                if event.state == ElementState::Pressed {
                    match &event.logical_key {
                        Key::Named(NamedKey::Escape) => context.window_target.unwrap().exit(),
                        Key::Character(val) if val == "q" => context.window_target.unwrap().exit(),
                        Key::Character(val) if val == "r" => self.reset(&mut context),
                        Key::Named(NamedKey::ArrowLeft) => self.do_move_player(Direction::Left),
                        Key::Named(NamedKey::ArrowRight) => self.do_move_player(Direction::Right),
                        Key::Named(NamedKey::ArrowUp) => self.do_move_player(Direction::Up),
                        Key::Named(NamedKey::ArrowDown) => self.do_move_player(Direction::Down),
                        _ => (),
                    }
                }
            }
        }
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
        scene_h: Handle<Scene>,
        _data: &[u8],
        context: &mut PluginContext,
    ) {
        let scene = context.scenes.try_get_mut(scene_h).unwrap();

        // TODO: mem::take ugly ?
        let LoadingState::WaitingScene(board) = mem::take(&mut self.board) else {
            panic!("Should be in loading state WaitingScene with a loaded board !");
        };

        let (width, height) = (board.width(), board.height());

        CameraBuilder::new(
            BaseBuilder::new().with_local_transform(
                TransformBuilder::new()
                    .with_local_position(Vector3::new(
                        ((width as f32) - 1.) / 2.,
                        ((height as f32) - 1.) / 2.,
                        -5.,
                    ))
                    .with_local_rotation(rotation())
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

        let mut crates = Vec::with_capacity(board.crates().len());

        let player = {
            let (i, j) = board.player();
            Self::create_rectangle(
                scene,
                material_for_player(images, self.direction),
                i,
                j,
                -0.,
            )
        };

        for j in 0..height {
            for i in 0..width {
                use CellKind::*;
                let BoardElem(movable, under) = board.get(i, j);
                match under {
                    Void => (),
                    Wall => {
                        Self::create_rectangle(scene, images.mur.clone(), i, j, 0.);
                    }
                    Floor => {
                        Self::create_rectangle(scene, images.sol.clone(), i, j, 0.);
                    }
                    Target => {
                        // TODO: il serait mieux d'enlever la transparence avec la couleur du sol ?
                        Self::create_rectangle(scene, images.sol.clone(), i, j, 0.);
                        Self::create_rectangle(scene, images.objectif.clone(), i, j, 0.);
                    }
                }

                if let Some(MovableItem::Crate(caisse)) = movable {
                    let (i, j) = caisse.pos();
                    crates.push(Self::create_rectangle(
                        scene,
                        material_for_crate(images, under),
                        i,
                        j,
                        0.,
                    ));
                }
            }
        }

        self.board = LoadingState::SceneFilled {
            board,
            scene: scene_h,
            player,
            crates,
        }
    }
}
