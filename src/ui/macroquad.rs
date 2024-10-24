//! Macroquad engine version
//!
//! This version provides it's own event loop.

use std::{error::Error, str::FromStr, time::Instant};

use macroquad::{prelude::*, Window};

use super::{Board, BoardElem, CellKind, Direction, MovableItem};

const ANIMATION_DURATION_MILIS: u64 = 200;

// Normally through a macro for main.
pub fn game_macroquad(level: &str) {
    Window::from_config(
        Conf {
            window_title: "Sokoban".to_owned(),
            ..Default::default()
        },
        game_macroquad_async(level.to_string()),
    );
}

async fn game_macroquad_async(level: String) {
    let mut state = State::new(level).await.unwrap();

    state.draw().unwrap();
    loop {
        state.resize_window_if_needed();
        if state.manage_input_and_should_quit() {
            break;
        }
        state.draw().unwrap();
        next_frame().await
    }
}

struct Images {
    caisse: Texture2D,
    caisse_ok: Texture2D,
    mario_bas: Texture2D,
    mario_droite: Texture2D,
    mario_gauche: Texture2D,
    mario_haut: Texture2D,
    mur: Texture2D,
    objectif: Texture2D,
}

struct State {
    /// Game state
    board: Board,
    /// Loaded images
    images: Images,
    /// Direction indicating where the caracting is facing
    direction: Direction,
    /// When the player moved last (for animation)
    last_move_instant: Instant,
    /// New position of the moved crated if any (for animation)
    moved_crate: Option<(u32, u32)>,
    // shader: Material,
}

struct ScaleInfos {
    img_w: f32,
    img_h: f32,
    tot_w: f32,
    tot_h: f32,
    scale_w: f32,
    scale_h: f32,
    win_w: f32,
    win_h: f32,
}

impl State {
    async fn new(level: String) -> Result<Self, Box<dyn Error>> {
        let state = State {
            board: Board::from_str(&level[..])?,
            images: Images {
                caisse: load_texture("images/caisse.jpg").await?,
                caisse_ok: load_texture("images/caisse_ok.jpg").await?,
                mario_bas: load_texture("images/mario_bas.gif").await?,
                mario_droite: load_texture("images/mario_droite.gif").await?,
                mario_gauche: load_texture("images/mario_gauche.gif").await?,
                mario_haut: load_texture("images/mario_haut.gif").await?,
                mur: load_texture("images/mur.jpg").await?,
                objectif: load_texture("images/objectif.png").await?,
            },
            direction: Direction::Down,
            last_move_instant: Instant::now(),
            moved_crate: None,
            // shader: load_material(
            //     ShaderSource::Glsl {
            //         fragment: MY_FRAGMENT_SHADER,
            //         vertex: DEFAULT_VERTEX_SHADER,
            //     },
            //     Default::default(),
            // )?,
        };

        Ok(state)
    }

    fn reset(&mut self) {
        self.board.reset();
        self.direction = Direction::Down;
        self.last_move_instant = Instant::now();
    }

    fn do_move_player(&mut self, dir: Direction) {
        if let Some(moved) = self.board.do_move_player(dir) {
            self.last_move_instant = Instant::now();
            self.moved_crate = moved;
        }
        self.direction = dir;
    }

    /// Calculates scale based on new window size.
    ///
    /// `win_resize` can contain the new size of the window, otherwise we get it from ctx.
    fn get_screen_scale(&self) -> ScaleInfos {
        let (img_w, img_h) = (self.images.mur.width(), self.images.mur.height());

        let (board_w, board_h) = (self.board.width() as f32, self.board.height() as f32);
        let (win_w, win_h) = (screen_width(), screen_height());
        let (tot_w, tot_h) = (board_w * img_w, board_h * img_h);
        let (scale_w, scale_h) = (win_w / tot_w, win_h / tot_h);

        ScaleInfos {
            img_w,
            img_h,
            tot_w,
            tot_h,
            scale_w,
            scale_h,
            win_w,
            win_h,
        }
    }

    pub fn draw(&self) -> Result<(), Box<dyn Error>> {
        let scale_infos = self.get_screen_scale();

        clear_background(BLACK);

        let scale = f32::min(scale_infos.scale_w, scale_infos.scale_h);

        let (mario, offset) = {
            let millis_since_last_move = Instant::now()
                .duration_since(self.last_move_instant)
                .as_millis() as f32;
            let ratio_move = scale
                * (1.
                    - f32::min(
                        1.,
                        millis_since_last_move / (ANIMATION_DURATION_MILIS as f32),
                    ));

            match self.direction {
                Direction::Up => (
                    &self.images.mario_haut,
                    (0., ratio_move * self.images.mur.height()),
                ),
                Direction::Down => (
                    &self.images.mario_bas,
                    (0., -ratio_move * self.images.mur.height()),
                ),
                Direction::Left => (
                    &self.images.mario_gauche,
                    (ratio_move * self.images.mur.width(), 0.),
                ),
                Direction::Right => (
                    &self.images.mario_droite,
                    (-ratio_move * self.images.mur.width(), 0.),
                ),
            }
        };

        // Apparently can't set it per draw (whole image has same texture parameter).
        {
            let filter = match self.direction {
                // Best for pixel art as it doesn't make things blurry.
                Direction::Up | Direction::Down => FilterMode::Nearest,
                Direction::Left | Direction::Right => FilterMode::Linear,
            };
            self.images.caisse.set_filter(filter);
            self.images.caisse_ok.set_filter(filter);
            self.images.mario_bas.set_filter(filter);
            self.images.mario_droite.set_filter(filter);
            self.images.mario_gauche.set_filter(filter);
            self.images.mario_haut.set_filter(filter);
            self.images.mur.set_filter(filter);
            self.images.objectif.set_filter(filter);
        }

        let mut foreground = [None, None];

        for j in 0..self.board.height() {
            // TODO: fix shader removing alpha
            // if j % 2 == 0 {
            //     gl_use_material(&self.shader);
            // } else {
            //     gl_use_default_material();
            // }
            for i in 0..self.board.width() {
                use CellKind::*;

                let (x, y) = (
                    i as f32 * scale_infos.img_w * scale,
                    j as f32 * scale_infos.img_h * scale,
                );

                let params = DrawTextureParams {
                    dest_size: Some(self.images.mur.size() * scale),
                    ..Default::default()
                };

                match self.board.get(i, j) {
                    BoardElem(_, Void) => (),
                    BoardElem(_, Wall) => draw_texture_ex(&self.images.mur, x, y, WHITE, params),
                    BoardElem(None, Floor) => draw_rectangle(
                        x,
                        y,
                        scale_infos.img_w * scale,
                        scale_infos.img_h * scale,
                        WHITE,
                    ),
                    BoardElem(None, Target) => {
                        draw_rectangle(
                            x,
                            y,
                            scale_infos.img_w * scale,
                            scale_infos.img_h * scale,
                            WHITE,
                        );
                        draw_texture_ex(&self.images.objectif, x, y, WHITE, params);
                    }
                    BoardElem(Some(movable), under) => {
                        match under {
                            Floor => draw_rectangle(
                                x,
                                y,
                                scale_infos.img_w * scale,
                                scale_infos.img_h * scale,
                                WHITE,
                            ),
                            Target => {
                                draw_rectangle(
                                    x,
                                    y,
                                    scale_infos.img_w * scale,
                                    scale_infos.img_h * scale,
                                    WHITE,
                                );
                                draw_texture_ex(&self.images.objectif, x, y, WHITE, params.clone());
                            }
                            Void | Wall => {
                                unreachable!("Mario can neither go on a wall or on the void.")
                            }
                        }

                        let image = match movable {
                            MovableItem::Player => mario,
                            MovableItem::Crate(_) if under == Target => &self.images.caisse_ok,
                            MovableItem::Crate(_) => &self.images.caisse,
                        };

                        let (offset_x, offset_y) = match movable {
                            MovableItem::Player => offset,
                            MovableItem::Crate(_) => self
                                .moved_crate
                                .filter(|(a, b)| (*a, *b) == (i, j))
                                .map_or((0., 0.), |_| offset),
                        };

                        let index = match movable {
                            MovableItem::Player => 0,
                            MovableItem::Crate(_) => 1,
                        };

                        if (offset_x, offset_y) != (0., 0.) {
                            foreground[index] = Some((image, x + offset_x, y + offset_y, params));
                        } else {
                            draw_texture_ex(image, x + offset_x, y + offset_y, WHITE, params);
                        }
                    }
                }
            }
        }

        if let Some((image, x, y, params)) = foreground[0].take() {
            draw_texture_ex(image, x, y, WHITE, params);
        }
        if let Some((image, x, y, params)) = foreground[1].take() {
            draw_texture_ex(image, x, y, WHITE, params);
        }

        // gl_use_default_material();

        {
            let fps_msg = format!("fps : {}", get_fps() as i32);
            let fps_dim = measure_text(&fps_msg[..], None, 21, 1.);
            draw_text(&fps_msg[..], 0., fps_dim.offset_y, 21., WHITE);
        }

        if self.board.has_won() {
            let won_msg_1 = "You won!";
            let won_msg_2 = "(Press Escape key to quit...)";

            let won_msg_1_measure = measure_text(won_msg_1, None, 21, 1.);
            let won_msg_2_measure = measure_text(won_msg_2, None, 21, 1.);
            let won_msg_w = f32::max(won_msg_1_measure.width, won_msg_2_measure.width);
            let won_msg_h = won_msg_1_measure.height + won_msg_2_measure.height;

            let margin = won_msg_h * 0.2;

            draw_rectangle(
                (scale_infos.win_w - won_msg_w) / 2. - margin * 2.,
                (scale_infos.win_h - won_msg_h) / 2. - margin * 4.,
                won_msg_w + margin * 4.,
                won_msg_h + margin * 8.,
                Color::from_rgba(150, 150, 0, 200),
            );

            draw_text(
                won_msg_1,
                (scale_infos.win_w - won_msg_1_measure.width) / 2.,
                scale_infos.win_h / 2. - margin - won_msg_1_measure.height
                    + won_msg_1_measure.offset_y,
                21.,
                BLACK,
            );
            draw_text(
                won_msg_2,
                (scale_infos.win_w - won_msg_2_measure.width) / 2.,
                scale_infos.win_h / 2. + margin + won_msg_2_measure.offset_y,
                21.,
                BLACK,
            );
        }

        Ok(())
    }

    /// Returns `true` if it should quit.
    pub fn manage_input_and_should_quit(&mut self) -> bool {
        if self.board.has_won() {
            is_key_pressed(KeyCode::Escape)
        } else {
            if is_key_pressed(KeyCode::R) {
                self.reset();
            }
            if is_key_pressed(KeyCode::Left) {
                self.do_move_player(Direction::Left);
            }
            if is_key_pressed(KeyCode::Right) {
                self.do_move_player(Direction::Right);
            }
            if is_key_pressed(KeyCode::Up) {
                self.do_move_player(Direction::Up);
            }
            if is_key_pressed(KeyCode::Down) {
                self.do_move_player(Direction::Down);
            }
            is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Q)
        }
    }

    fn resize_window_if_needed(&mut self) {
        let scale_infos = self.get_screen_scale();

        // To avoid unstable resize, we accept a small difference between w and h scales.
        if (scale_infos.scale_w * 10.).floor() != (scale_infos.scale_h * 10.).floor() {
            let scale = f32::min(scale_infos.scale_w, scale_infos.scale_h);
            let (new_width, new_height) = (scale_infos.tot_w * scale, scale_infos.tot_h * scale);

            if (new_width, new_height) != (scale_infos.win_w, scale_infos.win_h) {
                /*
                eprintln!(
                    "{new_width},{new_height} | {},{} | {},{}",
                    scale_infos.win_w,
                    scale_infos.win_h,
                    (scale_infos.scale_w * 10.).floor(),
                    (scale_infos.scale_h * 10.).floor()
                );
                */

                request_new_screen_size(new_width, new_height);
            }
        }
    }
}

// const MY_FRAGMENT_SHADER: &'static str = "#version 100
// precision lowp float;
//
// varying vec2 uv;
//
// uniform sampler2D Texture;
//
// void main() {
//     gl_FragColor = texture2D(Texture, uv);
// }
// ";
//
// const DEFAULT_VERTEX_SHADER: &'static str = "#version 100
// precision lowp float;
//
// attribute vec3 position;
// attribute vec2 texcoord;
//
// varying vec2 uv;
//
// uniform mat4 Model;
// uniform mat4 Projection;
//
// void main() {
//     gl_Position = Projection * Model * vec4(position, 1);
//     uv = texcoord;
// }
// ";

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
