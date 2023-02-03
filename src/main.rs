extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use opengl_graphics::OpenGL;
use opengl_graphics::GlGraphics;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow;

use std::collections::LinkedList;
use std::iter::FromIterator;

const WINDOW_SIZE: i32 = 1000;

struct Game {
    graphics: GlGraphics,
    snake: Snake,
    fruits: Vec<Fruit>,
    thread_rng: rand::rngs::ThreadRng,
    end_game: bool,
}

impl Game {
    fn render_event(&mut self, args: &RenderArgs) {
        let background: [f32; 4] = [0.7, 0.3, 0.2, 1.0];
        self.graphics.draw(args.viewport(), |_c, graphics| {
            graphics::clear(background, graphics);
        });

        self.snake.render(&mut self.graphics, &args);

        for fruit in self.fruits.iter() {
            fruit.render(&mut self.graphics, &args);
        }
    }

    fn update_event(&mut self) {
        self.snake.update();

        for i in 0..self.fruits.len() {
            if self.fruits[i].snake_can_eat(&self.snake) {
                self.fruits[i].set_rand_position(&mut self.thread_rng);
                self.snake.increase_body_size();
            }
        }

        if self.snake.snake_died() {
            self.end_game = true;
        }
    }

    fn pressed_event(&mut self, button: &Button) {
        let last_direction = self.snake.direction.clone();

        self.snake.direction = match button {
            &Button::Keyboard(Key::W) 
                if last_direction != SnakeDirection::Down => SnakeDirection::Up,
            &Button::Keyboard(Key::S) 
                if last_direction != SnakeDirection::Up => SnakeDirection::Down,
            &Button::Keyboard(Key::D) 
                if last_direction != SnakeDirection::Left => SnakeDirection::Right,
            &Button::Keyboard(Key::A) 
                if last_direction != SnakeDirection::Right => SnakeDirection::Left,
            _ => last_direction
        };
    }
}

#[derive(Clone, PartialEq)]
enum SnakeDirection {
    Right, Left, Up, Down,
}

struct Fruit {
    pos_x: i32,
    pos_y: i32,
}

impl Fruit {
    fn render(&self, graphics: &mut GlGraphics, args: &RenderArgs) {
        let color: [f32; 4] = [0.57, 0.79, 0.47, 1.0];

        let square = graphics::rectangle::square(
            (self.pos_x * (WINDOW_SIZE / 10)) as f64,
            (self.pos_y * (WINDOW_SIZE / 10)) as f64,
            (WINDOW_SIZE / 10) as f64
        );

        graphics.draw(args.viewport(), |c, graphics| {
            let transform = c.transform;

            graphics::rectangle(color, square, transform, graphics);
        });
    }

    fn snake_can_eat(&self, snake: &Snake) -> bool {
        if self.pos_x == snake.body.front().unwrap().0 {
            if self.pos_y == snake.body.front().unwrap().1 {
                return true;
            }
        }

        return false;
    }

    fn set_rand_position(&mut self, rng: &mut rand::rngs::ThreadRng) {
        use rand::Rng;

        let last_pos_x: i32 = self.pos_x;
        let last_pos_y: i32 = self.pos_y;
        let mut gen_new_pos: bool = true;

        while gen_new_pos {
            self.pos_x = rng.gen_range(0..9);
            self.pos_y = rng.gen_range(0..9);
            if last_pos_x != self.pos_x  && last_pos_y != self.pos_y{
                gen_new_pos = false;
            }
        }
    }
}

struct Snake {
    body: LinkedList<(i32, i32)>,
    direction: SnakeDirection,
}

impl Snake {
    fn increase_body_size(&mut self) {
       self.body.push_back((*self.body.back().expect("Snake has no back for some reason")).clone());
    }

    fn snake_died(&self) -> bool {
        let head: &(i32, i32) = self.body.front().expect("For some reason there is no head");

        let mut first: bool = true;
        for segment in self.body.iter() {
            if first {
                first = false;
                continue;
            }

            if segment.0 == head.0 {
                if segment.1 == head.1 {
                    return true;
                }
            }
        }

        return false;
    }

    fn render(&self, graphics: &mut GlGraphics, args: &RenderArgs) {
        let color: [f32; 4] = [0.96, 0.73, 0.25, 1.0];

        let squares: Vec<graphics::types::Rectangle> = self.body.iter().map(|&(x, y)| {
            graphics::rectangle::square(
                (x * (WINDOW_SIZE / 10)) as f64,
                (y * (WINDOW_SIZE / 10)) as f64,
                (WINDOW_SIZE / 10) as f64
            )
        }).collect();

        graphics.draw(args.viewport(), |c, graphics| {
            let transform = c.transform;
            squares.into_iter().for_each(|square| {
                graphics::rectangle(color, square, transform, graphics);
            });
        })
    }

    fn update(&mut self) {
        let mut new_head = (*self.body.front().expect("Snake has no Head")).clone();

        match self.direction {
            SnakeDirection::Right => new_head.0 += 1,
            SnakeDirection::Left => new_head.0 -= 1,
            SnakeDirection::Up => new_head.1 -= 1,
            SnakeDirection::Down => new_head.1 += 1,
        }

        if new_head.0 < 0 {
            new_head.0 = 9;
        }
        else if new_head.0 > 9 {
            new_head.0 = 0;
        }
        if new_head.1 < 0 {
            new_head.1 = 9;
        }
        else if new_head.1 > 9 {
            new_head.1 = 0;
        }

        self.body.push_front(new_head);
        self.body.pop_back().unwrap();
    }
}

fn main() {
    let opengl_instance = OpenGL::V3_2;
    let mut window: GlutinWindow = piston::window::WindowSettings::new(
        "Snake Game", [WINDOW_SIZE as u32, WINDOW_SIZE as u32]
    ).opengl(opengl_instance).exit_on_esc(true).build().unwrap();

    let mut game_instance = Game { 
        graphics: GlGraphics::new(opengl_instance), 
        snake: Snake { 
            body: LinkedList::from_iter(
                (vec![(0, 0), (0, 0)]).into_iter()
            ), 
            direction: SnakeDirection::Right 
        },
        fruits: vec!(Fruit { pos_x: 5, pos_y: 5 }),
        thread_rng: rand::thread_rng(),
        end_game: false
    };

    let mut event_system = Events::new(EventSettings::new()).ups(8);
    while let Some(event) = event_system.next(&mut window) {
        if !game_instance.end_game {
            if let Some(key) = event.button_args() {
                game_instance.pressed_event(&key.button);
            }

            if let Some(_update) = event.update_args() {
                game_instance.update_event();
            }
        }

        if let Some(renderer) = event.render_args() {
            game_instance.render_event(&renderer);
        }
    }
}