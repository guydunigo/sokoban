//! Ggez engine version
//!
//! This version provides it's own event loop.

use std::{env, path::PathBuf, str::FromStr};

use ggez::{
    conf::Conf,
    event,
    glam::Vec2,
    graphics::{self, Color, DrawMode, DrawParam, Drawable, Rect, Text},
    Context, ContextBuilder, GameError, GameResult,
};

use super::{Action, Board, BoardElem, CellKind, Direction, MovableItem};

pub fn game_ggez(level: &str) -> GameResult {
    // todo!("Window size");

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        println!("{manifest_dir}");
        let mut path = PathBuf::from(manifest_dir);
        path.push("images");
        path
    } else {
        todo!("check working ?");
        // PathBuf::from("/images")
    };

    let (ctx, event_loop) = ContextBuilder::new("Sokoban", "GuyDuNigo")
        .default_conf(Conf::default())
        .resources_dir_name(resource_dir)
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
    board: Board,
    images: Images,
    last_direction: Direction,
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
            last_direction: Direction::Down,
        };

        Ok(state)
    }
}

impl ggez::event::EventHandler<GameError> for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // todo!("register last action");
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);
        let dimensions = self
            .images
            .mur
            .dimensions(ctx)
            .expect("Can't get dimensions of wall picture !");

        let rect = new_rectangle(ctx, dimensions)?;

        let (board_w, board_h) = (self.board.width() as f32, self.board.height() as f32);
        let (screen_w, screen_h) = ctx.gfx.size();

        let (tot_w, tot_h) = (board_w * dimensions.w, board_h * dimensions.h);
        let (scale_w, scale_h) = (screen_w / tot_w, screen_h / tot_h);
        let min_scale = f32::min(scale_w, scale_h);

        let scale = Vec2::new(min_scale, min_scale);

        for j in 0..self.board.height() {
            for i in 0..self.board.width() {
                use CellKind::*;
                use MovableItem::*;

                let (x, y) = (
                    i as f32 * dimensions.w * min_scale,
                    j as f32 * dimensions.h * min_scale,
                );
                let params = DrawParam::default().dest(Vec2::new(x, y)).scale(scale);

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
                                canvas.draw(&self.images.objectif, params);
                            }
                            Void | Wall => {
                                unreachable!("Mario can neither go on a wall or on the void.")
                            }
                        }

                        let mario = match self.last_direction {
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
            }
        }
        canvas.draw(
            Text::new(format!("fps : {}", ctx.time.fps() as i32)).set_scale(32.),
            DrawParam::default().dest(Vec2::ZERO),
        );

        canvas.finish(ctx)?;

        Ok(())
    }
}

fn new_rectangle(ctx: &mut Context, bounding: Rect) -> GameResult<impl Drawable> {
    graphics::Mesh::new_rectangle(ctx, DrawMode::fill(), bounding, Color::WHITE)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
