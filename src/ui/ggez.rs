//! Ggez engine version
//!
//! This version provides it's own event loop.

use std::{env, path::PathBuf, str::FromStr};

use ggez::{
    conf::{Conf, WindowMode},
    event,
    glam::Vec2,
    graphics::{self, Color, DrawMode, DrawParam, Drawable, Rect, Text, TextAlign, TextLayout},
    input::keyboard::{KeyCode, KeyInput},
    Context, ContextBuilder, GameError, GameResult,
};

use super::{Board, BoardElem, CellKind, Direction, MovableItem};

pub fn game_ggez(level: &str) -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        println!("{manifest_dir}");
        let mut path = PathBuf::from(manifest_dir);
        path.push("images");
        path
    } else {
        PathBuf::from("/images")
    };

    let (ctx, event_loop) = ContextBuilder::new("Sokoban", "GuyDuNigo")
        .default_conf(Conf::default())
        .resources_dir_name(resource_dir)
        .window_mode(WindowMode {
            resizable: true,
            ..Default::default()
        })
        .build()
        .expect("Couldn't initialize context.");

    let state = State::new(&ctx, level)?;

    event::run(ctx, event_loop, state);
}

struct Images {
    caisse: graphics::Image,
    caisse_ok: graphics::Image,
    mario_bas: graphics::Image,
    mario_droite: graphics::Image,
    mario_gauche: graphics::Image,
    mario_haut: graphics::Image,
    mur: graphics::Image,
    objectif: graphics::Image,
}

struct State {
    /// Game state
    board: Board,
    /// Loaded images
    images: Images,
    /// Direction indicating where the caracting is facing
    direction: Direction,
}

struct ScaleInfos {
    dimensions: Rect,
    tot_w: f32,
    tot_h: f32,
    scale_w: f32,
    scale_h: f32,
    win_w: f32,
    win_h: f32,
}

impl State {
    fn new(ctx: &Context, level: &str) -> GameResult<Self> {
        let state = State {
            board: Board::from_str(level)
                .map_err(|e| GameError::CustomError(format!("Couldn't parse level : {e}")))?,
            images: Images {
                caisse: graphics::Image::from_path(ctx, "/caisse.jpg")?,
                caisse_ok: graphics::Image::from_path(ctx, "/caisse_ok.jpg")?,
                mario_bas: graphics::Image::from_path(ctx, "/mario_bas.gif")?,
                mario_droite: graphics::Image::from_path(ctx, "/mario_droite.gif")?,
                mario_gauche: graphics::Image::from_path(ctx, "/mario_gauche.gif")?,
                mario_haut: graphics::Image::from_path(ctx, "/mario_haut.gif")?,
                mur: graphics::Image::from_path(ctx, "/mur.jpg")?,
                objectif: graphics::Image::from_path(ctx, "/objectif.png")?,
            },
            direction: Direction::Down,
        };

        Ok(state)
    }

    fn do_move_player(&mut self, dir: Direction) {
        self.board.do_move_player(dir);
        self.direction = dir;
    }

    /// Calculates scale based on new window size.
    ///
    /// `win_resize` can contain the new size of the window, otherwise we get it from ctx.
    fn get_screen_scale(&self, ctx: &mut Context, win_resize: Option<(f32, f32)>) -> ScaleInfos {
        let dimensions = self
            .images
            .mur
            .dimensions(ctx)
            .expect("Can't get dimensions of wall picture !");

        let (board_w, board_h) = (self.board.width() as f32, self.board.height() as f32);
        let (win_w, win_h) = win_resize.unwrap_or(ctx.gfx.size());
        let (tot_w, tot_h) = (board_w * dimensions.w, board_h * dimensions.h);
        let (scale_w, scale_h) = (win_w / tot_w, win_h / tot_h);

        ScaleInfos {
            dimensions,
            tot_w,
            tot_h,
            scale_w,
            scale_h,
            win_w,
            win_h,
        }
    }
}

impl ggez::event::EventHandler<GameError> for State {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        let scale_infos = self.get_screen_scale(ctx, None);

        let rect = graphics::Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            scale_infos.dimensions,
            Color::WHITE,
        )?;

        let scale = f32::min(scale_infos.scale_w, scale_infos.scale_h);
        let scale_vec = Vec2::new(scale, scale);

        for j in 0..self.board.height() {
            for i in 0..self.board.width() {
                use CellKind::*;
                use MovableItem::*;

                if i % 2 == 0 {
                    // Best for pixel art as it doesn't make things blurry.
                    canvas.set_sampler(graphics::Sampler::nearest_clamp());
                }

                let (x, y) = (
                    i as f32 * scale_infos.dimensions.w * scale,
                    j as f32 * scale_infos.dimensions.h * scale,
                );
                let params = DrawParam::default().dest(Vec2::new(x, y)).scale(scale_vec);

                match self.board.get(i as isize, j as isize) {
                    BoardElem(_, Void) => (),
                    BoardElem(_, Wall) => canvas.draw(&self.images.mur, params),
                    BoardElem(None, Floor) => canvas.draw(&rect, params),
                    BoardElem(None, Target) => {
                        canvas.draw(&rect, params);
                        canvas.draw(&self.images.objectif, params);
                    }
                    BoardElem(Some(Player), under) => {
                        match under {
                            Floor => canvas.draw(&rect, params),
                            Target => {
                                canvas.draw(&rect, params);
                                canvas.draw(&self.images.objectif, params);
                            }
                            Void | Wall => {
                                unreachable!("Mario can neither go on a wall or on the void.")
                            }
                        }

                        let mario = match self.direction {
                            Direction::Up => &self.images.mario_haut,
                            Direction::Down => &self.images.mario_bas,
                            Direction::Left => &self.images.mario_gauche,
                            Direction::Right => &self.images.mario_droite,
                        };
                        canvas.draw(mario, params)
                    }
                    BoardElem(Some(Crate(_)), Floor) => canvas.draw(&self.images.caisse, params),
                    BoardElem(Some(Crate(_)), Target) => {
                        canvas.draw(&self.images.caisse_ok, params)
                    }
                }

                if i % 2 == 0 {
                    canvas.set_default_sampler();
                }
            }
        }
        canvas.draw(
            Text::new(format!("fps : {}", ctx.time.fps() as i32)).set_scale(32.),
            DrawParam::default().dest(Vec2::ZERO),
        );

        if self.board.has_won() {
            let mut won_msg = Text::new("You won!\n(Press any key to quit...)");
            won_msg.set_scale(32.);
            won_msg.set_layout(TextLayout {
                h_align: TextAlign::Middle,
                v_align: TextAlign::Begin,
            });

            let dest = Vec2::from(ctx.gfx.size()) / 2.;

            {
                let dimensions = won_msg
                    .dimensions(ctx)
                    .expect("Text should have dimensions !");

                let margin = dimensions.h * 0.1;
                let rect = Rect::new(
                    -(dimensions.w + margin) / 2.,
                    -(dimensions.h + margin) / 2.,
                    dimensions.w + margin,
                    dimensions.h + margin,
                );
                let won_box = graphics::Mesh::new_rectangle(
                    ctx,
                    DrawMode::fill(),
                    rect,
                    Color::from_rgba(150, 150, 0, 150),
                )?;
                canvas.draw(&won_box, DrawParam::default().dest(dest));
            }

            canvas.draw(
                &won_msg,
                DrawParam::default()
                    .dest(dest)
                    .offset(Vec2::new(0., 0.5))
                    .color(Color::BLACK),
            );
        }

        canvas.finish(ctx)?;

        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        if let Some(keycode) = input.keycode {
            match keycode {
                KeyCode::Escape | KeyCode::Q => ctx.request_quit(),
                KeyCode::R => self.board.reset(),
                KeyCode::Left => self.do_move_player(Direction::Left),
                KeyCode::Right => self.do_move_player(Direction::Right),
                KeyCode::Up => self.do_move_player(Direction::Up),
                KeyCode::Down => self.do_move_player(Direction::Down),
                _ => (),
            }
        }
        Ok(())
    }
    fn resize_event(&mut self, ctx: &mut Context, win_w: f32, win_h: f32) -> GameResult {
        let scale_infos = self.get_screen_scale(ctx, Some((win_w, win_h)));

        // To avoid unstable resize, we accept a small difference between w and h scales.
        if (scale_infos.scale_w * 100.).floor() != (scale_infos.scale_h * 100.).floor() {
            let scale = f32::min(scale_infos.scale_w, scale_infos.scale_h);
            let (new_width, new_height) = (scale_infos.tot_w * scale, scale_infos.tot_h * scale);

            if (new_width, new_height) != (scale_infos.win_w, scale_infos.win_h) {
                /*
                eprintln!(
                    "{new_width},{new_height} | {},{} | {},{}",
                    scale_infos.win_w,
                    scale_infos.win_h,
                    (scale_infos.scale_w * 100.).floor(),
                    (scale_infos.scale_h * 100.).floor()
                );
                */

                ctx.gfx.set_drawable_size(new_width, new_height)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
