//! Macroquad engine version
//!
//! This version provides it's own event loop.

use std::{error::Error, str::FromStr, time::Instant};

use macroquad::{prelude::*, Window};

use super::{Board, BoardElem, CellKind, Direction, MovableItem};

const ANIMATION_DURATION_MILIS: u64 = 200;

// TODO: gestion des erreurs

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
    let state = State::new(level).await.unwrap();

    loop {
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
    // shader: graphics::Shader,
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
                // TODO: set_filter(FilterMode::Nearest)
            },
            direction: Direction::Down,
            last_move_instant: Instant::now(),
            moved_crate: None,
            // TODO: shader
            // shader: graphics::ShaderBuilder::new()
            //     .fragment_path("/rand_noise_shader.wgsl")
            //     .build(&ctx.gfx)?,
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
    fn get_screen_scale(&self, win_resize: Option<(f32, f32)>) -> ScaleInfos {
        let (img_w, img_h) = (self.images.mur.width(), self.images.mur.height());

        let (board_w, board_h) = (self.board.width() as f32, self.board.height() as f32);
        let (win_w, win_h) = win_resize.unwrap_or_else(|| (screen_width(), screen_height()));
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
        let scale_infos = self.get_screen_scale(None);

        clear_background(BLACK);

        let scale = f32::min(scale_infos.scale_w, scale_infos.scale_h);

        let (mario, offset) = {
            let millis_since_last_move = Instant::now()
                .duration_since(self.last_move_instant)
                .as_millis() as f32;
            let ratio_move = 1.
                - f32::min(
                    1.,
                    millis_since_last_move / (ANIMATION_DURATION_MILIS as f32),
                );

            match self.direction {
                Direction::Up => (&self.images.mario_haut, (0., -ratio_move)),
                Direction::Down => (&self.images.mario_bas, (0., ratio_move)),
                Direction::Left => (&self.images.mario_gauche, (-ratio_move, 0.)),
                Direction::Right => (&self.images.mario_droite, (ratio_move, 0.)),
            }
        };

        for j in 0..self.board.height() {
            // TODO: if j % 2 == 0 {
            // TODO:     canvas.set_shader(&self.shader);
            // TODO: } else {
            // TODO:     canvas.set_default_shader();
            // TODO: }
            for i in 0..self.board.width() {
                use CellKind::*;

                // TODO: if i % 2 == 0 {
                // TODO:     // Best for pixel art as it doesn't make things blurry.
                // TODO:     canvas.set_sampler(graphics::Sampler::nearest_clamp());
                // TODO: }

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

                        draw_texture_ex(image, x + offset_x, y + offset_y, WHITE, params);
                    }
                }

                // TODO: if i % 2 == 0 {
                // TODO:     canvas.set_default_sampler();
                // TODO: }
            }
        }
        // TODO: canvas.set_default_shader();

        draw_text(
            &format!("fps : {}", get_fps() as i32)[..],
            10.,
            10.,
            21.,
            WHITE,
        );

        if self.board.has_won() {
            let won_msg_1 = "You won!";
            let won_msg_2 = "(Press Escape key to quit...)";

            let won_msg_1_measure = measure_text(won_msg_1, None, 21, 1.);
            let won_msg_2_measure = measure_text(won_msg_2, None, 21, 1.);
            let won_msg_w = f32::max(won_msg_1_measure.width, won_msg_2_measure.width);
            let won_msg_h = won_msg_1_measure.height + won_msg_2_measure.height;

            let margin = won_msg_h * 0.1;

            // TODO: Color::from_rgba(150, 150, 0, 200),
            draw_rectangle(
                (scale_infos.win_w - won_msg_w - margin) / 2.,
                (scale_infos.win_h - won_msg_h - margin) / 2.,
                won_msg_w + margin,
                won_msg_h + margin,
                WHITE,
            );

            draw_text(
                &won_msg_1,
                (scale_infos.win_w - won_msg_1_measure.width - margin) / 2.,
                (scale_infos.win_h - won_msg_h - margin) / 2. - won_msg_1_measure.height,
                21.,
                BLACK,
            );
            draw_text(
                &won_msg_2,
                (scale_infos.win_w - won_msg_2_measure.width - margin) / 2.,
                (scale_infos.win_h - won_msg_h - margin) / 2. - won_msg_2_measure.height,
                21.,
                BLACK,
            );
        }

        Ok(())
    }

    // TODO
    /*
    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        if let Some(keycode) = input.keycode {
            if self.board.has_won() {
                if keycode == KeyCode::Escape {
                    ctx.request_quit();
                }
            } else {
                match keycode {
                    KeyCode::Escape | KeyCode::Q => ctx.request_quit(),
                    KeyCode::R => self.reset(),
                    KeyCode::Left => self.do_move_player(Direction::Left),
                    KeyCode::Right => self.do_move_player(Direction::Right),
                    KeyCode::Up => self.do_move_player(Direction::Up),
                    KeyCode::Down => self.do_move_player(Direction::Down),
                    _ => (),
                }
            }
        }
        Ok(())
    }
    */

    /*
        // TODO
        fn resize_event(&mut self, ctx: &mut Context, win_w: f32, win_h: f32) -> GameResult {
            let scale_infos = self.get_screen_scale(ctx, Some((win_w, win_h)));

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

                    ctx.gfx.set_drawable_size(new_width, new_height)?;
                }
            }

            Ok(())
        }
    */
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
