//! Game project.
use fyrox::{
    asset::{untyped::ResourceKind, Resource},
    core::{
        algebra::{UnitQuaternion, Vector3},
        color::Color,
        math::curve::{Curve, CurveKey, CurveKeyKind},
        pool::Handle,
        reflect::prelude::*,
        visitor::prelude::*,
    },
    engine::GraphicsContext,
    event::{ElementState, Event, WindowEvent},
    generic_animation::{
        container::{TrackDataContainer, TrackValueKind},
        track::Track,
        value::ValueBinding,
        Animation,
    },
    gui::{
        border::BorderBuilder,
        brush::Brush,
        formatted_text::WrapMode,
        grid::{Column, GridBuilder, Row},
        message::{MessageDirection, UiMessage},
        screen::ScreenBuilder,
        text::{TextBuilder, TextMessage},
        widget::WidgetBuilder,
        HorizontalAlignment, Thickness, UiNode,
    },
    keyboard::{Key, NamedKey},
    material::{Material, MaterialResource},
    plugin::{Plugin, PluginContext, PluginRegistrationContext},
    renderer::QualitySettings,
    resource::texture::{Texture, TextureMagnificationFilter, TextureMinificationFilter},
    scene::{
        animation::{AnimationContainer, AnimationPlayer, AnimationPlayerBuilder},
        base::BaseBuilder,
        camera::{CameraBuilder, OrthographicProjection, Projection, SkyBox},
        dim2::rectangle::{Rectangle, RectangleBuilder},
        graph::Graph,
        node::Node,
        transform::TransformBuilder,
        Scene,
    },
    window::Fullscreen,
};
use sokoban::{Board, BoardElem, CellKind, Crate, Direction};
use std::{fs::read_to_string, mem, path::Path, str::FromStr};

const DEFAULT_LEVEL_FILENAME: &str = "../map.txt";
const ANIMATION_DURATION: f32 = 0.2;

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

fn material_for_crate(images: &Images, board: &Board, this_crate: &Crate) -> MaterialResource {
    if this_crate.is_placed(board) {
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
enum LoadingState {
    #[default]
    None,
    WaitingScene(Board, Images),
    SceneFilled {
        images: Images,
        board: Board,
        scene: Handle<Scene>,
        player: Handle<Node>,
        crates: Vec<Handle<Node>>,
        animation_player: Handle<Node>,
        fps: Handle<UiNode>,
    },
    Won,
}

impl LoadingState {
    fn unwrap_scene_filled(
        &mut self,
    ) -> (
        &Images,
        &mut Board,
        &Handle<Scene>,
        &Handle<Node>,
        &[Handle<Node>],
        &Handle<Node>,
        &Handle<UiNode>,
    ) {
        if let LoadingState::SceneFilled {
            images,
            board,
            scene,
            player,
            crates,
            animation_player,
            fps,
        } = self
        {
            (
                images,
                board,
                scene,
                player,
                &crates[..],
                animation_player,
                fps,
            )
        } else {
            panic!("Game should be in LoadingStata::SceneFilled with all the board loaded into the scene !");
        }
    }
}

#[derive(Default, Visit, Reflect, Debug)]
pub struct Game {
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

    fn reset_animations(
        graph: &mut Graph,
        animation_player: Handle<Node>,
    ) -> (&mut AnimationContainer, Handle<Animation<Handle<Node>>>) {
        let animation_player: &mut AnimationPlayer = graph[animation_player].cast_mut().unwrap();
        let animations = animation_player.animations_mut();
        // Clean up finished animations
        animations.retain(|a| !a.has_ended());

        let handle = animations.add(Self::new_animation());
        (animations, handle)
    }

    fn new_animation() -> Animation<Handle<Node>> {
        let mut animation = Animation::default();
        animation.set_time_slice(0.0..ANIMATION_DURATION);
        animation.set_loop(false);
        animation.set_enabled(true);
        animation
    }

    fn add_animation(
        animations: &mut AnimationContainer,
        animation: Handle<Animation<Handle<Node>>>,
        node: Handle<Node>,
        dir: Direction,
        dst: (u32, u32),
    ) {
        // If another track in a running animation also acts on our the node, we disable it to avoid conflicts :
        animations.iter_mut().for_each(|a| {
            a.tracks_mut()
                .iter_mut()
                .filter(|t| t.target() == node)
                .for_each(|t| t.set_enabled(false));
        });

        let animation = &mut animations[animation];

        use Direction::*;
        let (xyz, src, dst, other) = match dir {
            Up => (1, dst.1 + 1, dst.1, dst.0),
            Down => (1, dst.1 - 1, dst.1, dst.0),
            Left => (0, dst.0 + 1, dst.0, dst.1),
            Right => (0, dst.0 - 1, dst.0, dst.1),
        };

        let mut frames_container = TrackDataContainer::new(TrackValueKind::Vector3);
        // We'll animate only X coordinate (at index 0).
        frames_container.curves_mut()[xyz] = Curve::from(vec![
            CurveKey::new(0.0, src as f32, CurveKeyKind::Linear),
            CurveKey::new(ANIMATION_DURATION, dst as f32, CurveKeyKind::Linear),
        ]);
        frames_container.curves_mut()[1 - xyz] = Curve::from(vec![CurveKey::new(
            0.0,
            other as f32,
            CurveKeyKind::Constant,
        )]);
        // Create a track that will animated the node using the curve above.
        let mut track = Track::new(frames_container, ValueBinding::Position);
        track.set_target(node);

        animation.add_track(track);
    }

    fn reset(&mut self, context: &mut PluginContext) {
        let (images, board, scene, player, crates, animation_player, _) =
            self.board.unwrap_scene_filled();
        board.reset();

        self.direction = Direction::default();

        let graph = &mut context.scenes.try_get_mut(*scene).unwrap().graph;

        let (animations, animation) = Self::reset_animations(graph, *animation_player);

        // Self::update_node_pos(&mut graph, *player, board.player());
        Self::add_animation(
            animations,
            animation,
            *player,
            self.direction,
            board.player(),
        );

        crates.iter().zip(board.crates()).for_each(|(h, c)|
            // Self::update_node_pos(graph, *h, c.pos())
            Self::add_animation(animations, animation, *h, self.direction, c.pos()));

        graph[*player]
            .cast_mut::<Rectangle>()
            .unwrap()
            .material_mut()
            .set_value_and_mark_modified(material_for_player(images, self.direction));
        crates.iter().zip(board.crates()).for_each(|(h, c)| {
            graph[*h]
                .cast_mut::<Rectangle>()
                .unwrap()
                .material_mut()
                .set_value_and_mark_modified(material_for_crate(images, board, c));
        });
    }

    /*
    fn update_node_pos(graph: &mut Graph, node: Handle<Node>, (i, j): (u32, u32)) {
        let current_transform = graph[node].local_transform_mut();
        current_transform.set_position(Vector3::new(
            i as f32,
            j as f32,
            current_transform.position().z,
        ));
    }
    */

    fn do_move_player(&mut self, context: &mut PluginContext, dir: Direction) {
        let (images, board, scene, player, crates, animation_player, _) =
            self.board.unwrap_scene_filled();

        let graph = &mut context.scenes.try_get_mut(*scene).unwrap().graph;
        graph[*player]
            .cast_mut::<Rectangle>()
            .unwrap()
            .material_mut()
            .set_value_and_mark_modified(material_for_player(images, dir));

        if let Some(moved_crate_index) = board.do_move_player(dir) {
            let (animations, animation) = Self::reset_animations(graph, *animation_player);

            // Self::update_node_pos(graph, *player, board.player());
            Self::add_animation(animations, animation, *player, dir, board.player());

            if let Some(moved_crate_index) = moved_crate_index {
                let moved_crate = board.crates()[moved_crate_index];
                let crate_rect = crates[moved_crate_index];

                Self::add_animation(animations, animation, crate_rect, dir, moved_crate.pos());
                // Self::update_node_pos(graph, crates[moved_crate_index], moved_crate.pos());

                graph[crate_rect]
                    .cast_mut::<Rectangle>()
                    .unwrap()
                    .material_mut()
                    .set_value_and_mark_modified(material_for_crate(images, board, &moved_crate));

                if board.has_won() {
                    let _ = mem::replace(&mut self.board, LoadingState::Won);
                    let ui = context.user_interfaces.first_mut();
                    let text =
                        TextBuilder::new(WidgetBuilder::new().with_margin(Thickness::uniform(20.)))
                            .with_horizontal_text_alignment(HorizontalAlignment::Center)
                            .with_text("You won!\n(Press Escape key to quit...)")
                            .with_wrap(WrapMode::Word)
                            .with_font_size(21.)
                            .build(&mut ui.build_ctx());
                    let border = BorderBuilder::new(
                        WidgetBuilder::new()
                            .with_child(text)
                            .on_row(1)
                            .on_column(1)
                            .with_background(Brush::Solid(Color::from_rgba(150, 150, 0, 200))),
                    )
                    .with_corner_radius(20.)
                    .with_stroke_thickness(Thickness::uniform(0.))
                    .build(&mut ui.build_ctx());

                    ScreenBuilder::new(
                        WidgetBuilder::new().with_child(
                            GridBuilder::new(
                                WidgetBuilder::new()
                                    .with_width(300.0)
                                    .with_height(400.0)
                                    .with_child(border),
                            )
                            // Split the grid into 3 rows and 3 columns. The center cell contain the stack panel
                            // instance, that basically stacks main menu buttons one on top of another. The center
                            // cell will also be always centered in screen bounds.
                            .add_row(Row::stretch())
                            .add_row(Row::auto())
                            .add_row(Row::stretch())
                            .add_column(Column::stretch())
                            .add_column(Column::auto())
                            .add_column(Column::stretch())
                            .build(&mut ui.build_ctx()),
                        ),
                    )
                    .build(&mut ui.build_ctx());
                }
            }
        }

        self.direction = dir;
    }

    fn toggle_fullscreen(context: PluginContext) {
        if let GraphicsContext::Initialized(ref graphics_context) = context.graphics_context {
            if graphics_context.window.fullscreen().is_none() {
                /*
                graphics_context
                    .window
                    .set_fullscreen(Some(Fullscreen::Borderless(None)));
                */
                if let Some(monitor) = graphics_context.window.current_monitor() {
                    if let Some(first_available_video_mode) = monitor.video_modes().next() {
                        graphics_context
                            .window
                            .set_fullscreen(Some(Fullscreen::Exclusive(
                                first_available_video_mode,
                            )));
                    }
                }
            } else {
                graphics_context.window.set_fullscreen(None);
            }
        }
    }

    fn cycle_quality(context: PluginContext) {
        if let GraphicsContext::Initialized(ref mut graphics_context) = context.graphics_context {
            let settings = graphics_context.renderer.get_quality_settings();
            let next_settings = if settings == QualitySettings::low() {
                QualitySettings::medium()
            } else if settings == QualitySettings::medium() {
                QualitySettings::high()
            } else {
                QualitySettings::low()
            };
            let _ = graphics_context
                .renderer
                .set_quality_settings(&next_settings);
        }
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
            let arg1 = std::env::var("SOKOBAN_LEVEL");
            let level_filename = arg1.as_ref().map_or(DEFAULT_LEVEL_FILENAME, |f| &f[..]);

            let level = read_to_string(level_filename)
                .expect(&format!("Could not open file `{level_filename}`.")[..]);

            Board::from_str(&level[..]).expect("Failed to load level !")
        };

        self.board = LoadingState::WaitingScene(board, Images::load(&mut context));
    }

    fn on_deinit(&mut self, _context: PluginContext) {
        // Do a cleanup here.
    }

    fn update(&mut self, context: &mut PluginContext) {
        // Add your global update code here.
        if !matches!(self.board, LoadingState::Won) {
            let (_, _, _, _, _, _, fps) = self.board.unwrap_scene_filled();

            let frames_per_second = if let GraphicsContext::Initialized(ref graphics_context) =
                context.graphics_context
            {
                graphics_context.renderer.get_statistics().frames_per_second
            } else {
                0
            };

            let quality = if let GraphicsContext::Initialized(ref mut graphics_context) =
                context.graphics_context
            {
                let settings = graphics_context.renderer.get_quality_settings();
                if settings == QualitySettings::low() {
                    "low"
                } else if settings == QualitySettings::medium() {
                    "medium"
                } else {
                    "high"
                }
            } else {
                "unknown"
            };

            context
                .user_interfaces
                .first_mut()
                .send_message(TextMessage::text(
                    *fps,
                    MessageDirection::ToWidget,
                    format!(
                        "fps | loop {} | render {} | settings {}",
                        f32::round(1. / context.dt),
                        frames_per_second,
                        quality
                    ),
                ));
        }
    }

    fn on_os_event(&mut self, event: &Event<()>, mut context: PluginContext) {
        // Do something on OS event here.
        if let Event::WindowEvent {
            event: WindowEvent::KeyboardInput { event, .. },
            ..
        } = event
        {
            if event.state == ElementState::Pressed {
                if matches!(self.board, LoadingState::Won) {
                    if matches!(&event.logical_key, Key::Named(NamedKey::Escape)) {
                        context.window_target.unwrap().exit();
                    }
                } else {
                    match &event.logical_key {
                        Key::Named(NamedKey::Escape) => context.window_target.unwrap().exit(),
                        Key::Character(val) if val == "q" => context.window_target.unwrap().exit(),
                        Key::Character(val) if val == "r" => self.reset(&mut context),
                        Key::Character(val) if val == "f" => Self::toggle_fullscreen(context),
                        Key::Character(val) if val == "g" => Self::cycle_quality(context),
                        Key::Named(NamedKey::ArrowLeft) => {
                            self.do_move_player(&mut context, Direction::Left)
                        }
                        Key::Named(NamedKey::ArrowRight) => {
                            self.do_move_player(&mut context, Direction::Right)
                        }
                        Key::Named(NamedKey::ArrowUp) => {
                            self.do_move_player(&mut context, Direction::Up)
                        }
                        Key::Named(NamedKey::ArrowDown) => {
                            self.do_move_player(&mut context, Direction::Down)
                        }
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

        let LoadingState::WaitingScene(board, images) = mem::take(&mut self.board) else {
            panic!("Should be in loading state WaitingScene with a loaded board and images !");
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

        let mut animations = AnimationContainer::new();
        let animation = animations.add(Self::new_animation());

        let player = {
            let (i, j) = board.player();
            Self::create_rectangle(
                scene,
                material_for_player(&images, self.direction),
                i,
                j,
                -0.,
            )
        };
        Self::add_animation(
            &mut animations,
            animation,
            player,
            Direction::default(),
            board.player(),
        );

        let crates = board
            .crates()
            .iter()
            .map(|c| {
                let (i, j) = c.pos();
                let ch =
                    Self::create_rectangle(scene, material_for_crate(&images, &board, c), i, j, 0.);
                Self::add_animation(
                    &mut animations,
                    animation,
                    ch,
                    Direction::default(),
                    c.pos(),
                );
                ch
            })
            .collect();

        for j in 0..height {
            for i in 0..width {
                use CellKind::*;
                let BoardElem(_, under) = board.get(i, j);
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
            }
        }

        let animation_player = AnimationPlayerBuilder::new(BaseBuilder::new())
            .with_animations(animations)
            .build(&mut scene.graph);

        let fps = TextBuilder::new(WidgetBuilder::new())
            .with_text("fps : XX")
            .build(&mut context.user_interfaces.first_mut().build_ctx());

        self.board = LoadingState::SceneFilled {
            images,
            board,
            scene: scene_h,
            player,
            crates,
            animation_player,
            fps,
        }
    }
}
