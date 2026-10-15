#[allow(unused_imports)]
use ggez::{
    event::{self, EventHandler}, 
    graphics::{self, Color, Canvas, Mesh, Image, DrawParam}, 
    conf::{WindowMode, FullscreenType},
    input::{self, keyboard::KeyCode, keyboard::KeyboardContext},
    glam::{Vec2, vec2},
    Context, 
    GameResult,
    ContextBuilder,
};
use rand::{Rng, distributions::uniform::SampleUniform};
use std::{cell::Cell, collections::HashMap, println, rc::Rc, vec};
use std::time::{Duration, Instant};

const RADIUS_MAX: f32 = 60.0;
const RADIUS_MIN: f32 = 35.0;
const NUMOFBALLS: i32 = 20;
const GRAVITY: f32 = 0.2;
const FRICTION: f32 = 0.01;
const BOUNCEFRICTION: f32 = 0.99;

struct MainState {
    balls: Vec<Rc<Ball>>,
    collision_handler: CollisionHandler,
    zones: Vec<Zone>,
    scale_factor: f32,
    win_dim: Vec2,
}

struct Zone{
    pos: Vec2,
    size: Vec2,
    index: i32,
    balls: Vec<Rc<Ball>>,
}

struct Ball {
    acc: Cell<Vec2>,
    vel: Cell<Vec2>,
    pos: Cell<Vec2>,
    radius: f32,
    mass: f32,
    color: Color,
    id: i32,
    zone: Cell<[usize; 2]>,
}

impl Ball {
    fn update(&self) {
        let mut vel = self.vel.get();
        let mut pos = self.pos.get();
        let mut acc = self.acc.get();

        self.join_zone();


        // gravity
        acc += vec2(0.0, GRAVITY);

        // physics
        acc -= vel * FRICTION;
        vel += acc;
        pos += vel;

        self.acc.set(vec2(0.0, 0.0));

        self.vel.set(vel);
        self.pos.set(pos);

    }

    fn wall_collision(&self, win_dim: Vec2) {
        let mut vel = self.vel.get();
        let mut pos = self.pos.get();

        if pos.x - self.radius < 0.0 {
            pos.x = self.radius;
            vel.x *= -BOUNCEFRICTION;
        } else if pos.x + self.radius > win_dim.x {
            pos.x = win_dim.x - self.radius;
            vel.x *= -BOUNCEFRICTION;
        }

        if pos.y - self.radius < 0.0 {
            pos.y = self.radius;
            vel.y *= -BOUNCEFRICTION;
        } else if pos.y + self.radius > win_dim.y {
            pos.y = win_dim.y - self.radius;
            vel.y *= -BOUNCEFRICTION;
        }

        self.vel.set(vel);
        self.pos.set(pos);

    }
    
    fn ball_collision(&self, ball: &Ball) {
        // CELL requires to take value out before setting again with .set()
        let v1 = self.vel.get();
        let v2 = ball.vel.get();

        let x1 = self.pos.get();
        let x2 = ball.pos.get();

        let m1 = self.mass;
        let m2 = ball.mass;

        let v1n = v1 - (2.0 * m2)/(m1 + m2) * ((v1 - v2).dot(x1 - x2) / (x1 - x2).length_squared()) * (x1 - x2);
        let v2n = v2 - (2.0 * m1)/(m2 + m1) * ((v2 - v1).dot(x2 - x1) / (x2 - x1).length_squared()) * (x2 - x1);

        self.vel.set(v1n);
        ball.vel.set(v2n);
    }

    fn fix_pos(&self, ball: &Ball) {
        let mut pos1 = self.pos.get();
        let mut pos2 = ball.pos.get();

        if pos1.distance(pos2) < self.radius + ball.radius + 5.0 { 
            let mut dir = pos1 - pos2;
            dir = dir.normalize();
            pos1 += dir * 0.1;
            pos2 -= dir * 0.1;
        }

        self.pos.set(pos1);
        ball.pos.set(pos2);
    }

    fn draw(&self, ctx: &mut Context, canvas: &mut Canvas) {
        let mesh = Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            self.pos.get(),
            self.radius,
            0.1,
            self.color,
        ).unwrap();

        canvas.draw(&mesh, Vec2::ZERO);
    }

    fn join_zone(&self) {
        let pos = self.pos.get();

        let zone: [usize; 2] = [(pos.x / self.radius) as usize, (pos.y / self.radius) as usize];

        self.zone.set(zone);
        
        self.pos.set(pos);
    }
}

fn random<T: SampleUniform + PartialOrd>(min: T, max: T) -> T {
    if min >= max {
        return min;
    }
    rand::thread_rng().gen_range(min..max)
}

struct CollisionHandler {
    // collision_hash: HashMap<i32, i32>
}
impl CollisionHandler {
    // OLD FAULTY CODE
    // fn collide_balls(&mut self, ball_vec: &Vec<Rc<Ball>>, zones: &mut Vec<Zone>) {
    //     self.collision_hash.clear();
    //     for ball0 in ball_vec {
    //         for ball1 in ball_vec {
    //             if ball0.id == ball1.id { continue; }
    //             if self.collision_hash.contains_key(&ball1.id) {
    //                 if self.collision_hash[&ball1.id] == ball0.id { continue; }
    //             }
    //             if self.collision_hash.contains_key(&ball0.id) {
    //                 if self.collision_hash[&ball0.id] == ball1.id { continue; }
    //             }

    //             if ball0.pos.get().distance(ball1.pos.get()) > ball0.radius + ball1.radius { continue; }
    //             self.collision_hash.insert(ball0.id, ball1.id);
                
    //             ball0.ball_collision(&ball1);
    //             ball0.fix_pos(&ball1);
                
    //         }
    //     }
    // }

    fn new_collide_balls(&mut self, ball_vec: &Vec<Rc<Ball>>) {
        for i in 0..ball_vec.len() {
            for j in (i + 1)..ball_vec.len() {
                let (a, b) = (&ball_vec[i], &ball_vec[j]);

                let (vel_a, vel_b) = (a.vel.get(), b.vel.get());
                let (pos_a, pos_b) = (a.pos.get(), b.pos.get());

                let square_dist = pos_a.distance_squared(pos_b);
                if square_dist > a.radius * b.radius * 4.0 { continue; }

                a.fix_pos(b);
                
                if (vel_a - vel_b).dot(pos_a - pos_b) > 0.0 { continue; }
                
                a.ball_collision(b);
            }
        }
    }
}

// ------------------ BALL ------------------




impl MainState {
    fn spawn_balls(&mut self, color_vec: Vec<Color>) {
        for i in 0..NUMOFBALLS {
            let color_index = random(0, 4);
            // let radius = RADIUS * random(0.70, 1.30);
            let radius = random(RADIUS_MIN, RADIUS_MAX);
            self.balls.push(
                Rc::new(Ball {
                    acc: Cell::new(vec2(0.0, 0.0)),
                    vel: Cell::new(vec2(random(-5.0, 5.0), 0.0)),
                    pos: Cell::new(vec2(random(radius, self.win_dim.x), random(0.0, self.win_dim.y))),
                    radius,
                    mass: radius,
                    color: color_vec[color_index],
                    id: i,
                    zone: Cell::new([0, 0]),
                })
            );
        }
    }
}

// ------------------ ZONE ------------------

impl MainState {
    fn spawn_zones(&mut self) {
        let size = RADIUS_MAX;
        let mut i: i32 = 0;

        for x in (0..self.win_dim.x as i32).step_by(size as usize) {
            for y in (0..self.win_dim.y as i32).step_by(size as usize) {
                self.zones.push(
                    Zone {
                        pos: vec2(x as f32, y as f32),
                        size: vec2(size, size),
                        index: i,
                        balls: vec![],
                    }
                );
                i += 1;
            }
        }
    }

    fn get_balls(&mut self) {
        for ball in &self.balls {
            for zone in &mut self.zones {
                if ball.pos.get().x > zone.pos.x && ball.pos.get().x < zone.pos.x + zone.size.x {
                    if ball.pos.get().y > zone.pos.y && ball.pos.get().y < zone.pos.y + zone.size.y {
                        zone.balls.push(Rc::clone(ball));
                    }
                }
            }
        }
    }
}

// ------------------ GGEZ CONFIG ------------------

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if ctx.keyboard.is_key_pressed(KeyCode::Space) {
            self.balls.clear();
            self.spawn_balls(create_color_vec());
        }

        if ctx.keyboard.is_key_just_pressed(KeyCode::A) {
            self.spawn_balls(create_color_vec());
        }
        
        for ball in &mut self.balls {
            ball.update();
            ball.wall_collision(self.win_dim);
        }

        self.get_balls();

        self.collision_handler.new_collide_balls(&self.balls);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);

        for ball in &self.balls {
            ball.draw(ctx, &mut canvas);
        }

        canvas.finish(ctx)
    }
}

fn create_color_vec() -> Vec<Color> {

    let color_vec = vec![
        Color::from_rgb_u32(0xdad7cd),
        Color::from_rgb_u32(0xa3b18a),
        Color::from_rgb_u32(0x588157),
        Color::from_rgb_u32(0x3a5a40),
        Color::from_rgb_u32(0x344e41),
    ];

    color_vec
}

fn main() {
    
    let cb = ContextBuilder::new("first_draw", "Kasper Larsson-Skauli")
    .window_mode(WindowMode::default().maximized(true).fullscreen_type(FullscreenType::Desktop));
    // .window_mode(WindowMode::default().maximized(false).fullscreen_type(FullscreenType::Windowed).dimensions(700.0, 500.0));

    let (ctx, event_loop) = cb.build().unwrap();

    let win_dim: Vec2 = ctx.gfx.size().into();

    let scale_factor = win_dim.x / RADIUS_MAX as f32;

    // ------------------ State ------------------




    let mut state = MainState {
        balls: vec![],
        scale_factor,
        win_dim,
        collision_handler: CollisionHandler {},
        zones: vec![],
    };

    state.spawn_balls(
        create_color_vec()
    );
    state.spawn_zones();

    event::run(ctx, event_loop, state);
}