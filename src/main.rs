extern crate cgmath;
extern crate rand;
extern crate termion;
extern crate rayon;
use cgmath::Vector2;
use cgmath::InnerSpace;
use cgmath::MetricSpace;
// use rand::{task_rng, Rng};
use rand::Rng;

use std::f32::consts::PI;

use rayon::prelude::*;

use termion::{clear, cursor, style};
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use termion::event::Key;

use std::env;
use std::io::{self, Read, Write};
use std::process;

const BOID: &'static str = "o";


const width: f32 = 300.0;
const height: f32 = 80.0;


// pub fn main() {
//     let mut flock = Flock::new();
//     for _ in 0..150 {
//         flock.add_boid(Boid::new(width/2.0,height/2.0));
//     }
//     loop {
//         flock.run();
//         for b in flock.boids.iter() {
//             println!("x:{} y:{}", b.position.x, b.position.y);
//         }
//     }
// }

pub fn main() {
    let stdout = io::stdout();
    // let mut stdout = stdout.lock();
    let stdin = io::stdin();
    // let stdin = stdin.lock();
    let stderr = io::stderr();
    // let mut stderr = stderr.lock();


    // We go to raw mode to make the control over the terminal more fine-grained.
    // let stdout = stdout.into_raw_mode().unwrap();

    let termsize = termion::terminal_size().ok();
    let termwidth = termsize.map(|(w,_)| w - 2);
    let termheight = termsize.map(|(_,h)| h - 2);

    init(stdout, stdin, termwidth.unwrap(), termheight.unwrap());
}


fn start<W: Write, R: Read>(mut stdout: W, stdin: R, mut flock: Flock, w: u16, h: u16) {
    let mut last_update = std::time::Instant::now();
    // let sixteen_ms = std::time::Duration::from_millis(16); // 60 FPS
    let sixteen_ms = std::time::Duration::from_millis(28); // 30 FPS
    loop {
        let duration_since_last_update = std::time::Instant::now().duration_since(last_update);
        if duration_since_last_update < sixteen_ms {
            std::thread::sleep(sixteen_ms - duration_since_last_update);
        }
        last_update = std::time::Instant::now();

        write!(stdout, "{}", clear::All).unwrap();
        flock.run();
        stdout.flush().unwrap();

        for b in flock.boids.iter() {
            if b.pos_x() > 0 && b.pos_y() > 0 && b.pos_x() < w && b.pos_y() < h {
                write!(stdout, "{}", cursor::Goto(b.pos_x(), b.pos_y())).unwrap();
                // write!(stdout, "{}", termion::color::Fg(termion::color::Rgb(( b.vel_mag() / 1.5 * 255.0 ) as u8, 0.0 as u8, 0.0 as u8)));
                write!(stdout, "{}", termion::color::Fg(termion::color::Rgb(255.0 as u8, ( b.velocity.x / 3.0 * 255.0 ) as u8, ( b.velocity.y / 3.0 * 255.0 ) as u8)));


                stdout.write(b.render().as_bytes()).unwrap();
            }
        }


        // for b in flock.boids.iter() {
        //     // println!("x:{} y:{}", b.position.x, b.position.y);
        //     // println!("x:{} y:{}", b.pos_x(), b.pos_y());
        //     write!(stdout, "{}", cursor::Goto(b.pos_x(), b.pos_y())).unwrap();
        //     stdout.write(b.render().as_bytes()).unwrap();
        // }
    }

}

fn init<W: Write, R: Read>(mut stdout: W, stdin: R, w: u16, h: u16) {
    // write!(stdout, "{}", clear::All).unwrap();

    // Set the initial game state.
    let mut flock = Flock::new();
    // Add an initial set of boids into the system
    for _ in 0..1000 {
        flock.add_boid(Boid::new(( w as f32)/2.0,( h as f32 )/2.0, w as f32, h as f32));
    }
    // flock.add_boid(Boid::new(width/2.0,height/2.0));

    // Reset that game.
    // game.reset();
    // write!(stdout, "{}", clear::All).unwrap();

    // Start the event loop.
    start(stdout, stdin, flock, w, h);
}
// fn draw() {
//   background(50);
//   flock.run();
// }

// Add a new boid into the System
// fn mousePressed() {
//   flock.addBoid(new Boid(mouseX,mouseY));
// }



// The Flock (a list of Boid objects)
struct Flock {
    boids: Vec<Boid>
}

impl Flock {

    pub fn new() -> Flock {
        Flock{boids: vec![]}
    }


    pub fn run(&mut self) {
        let mut clone_boids = self.boids.clone();
        let clone_boids_other = clone_boids.split_off(200);

        self.boids.par_iter_mut().for_each(|b| {
            ( *b ).run(&clone_boids);
            // println!("{:?}", b);
        });
    }

    pub fn add_boid(&mut self, b: Boid) {
        self.boids.push(b);
    }
}



#[derive(Clone, Debug)]
struct Boid {
    position: Vector2<f32>,
    velocity: Vector2<f32>,
    acceleration: Vector2<f32>,
    r: f32,
    max_force: f32,
    max_speed: f32,
    width: f32,
    height: f32
}


impl Boid {
    fn new(x: f32, y: f32, w: f32, h: f32) -> Boid {
        let angle: f32 = rand::thread_rng().gen_range(0.0, 2.0 * PI);

        // let offset: f32 = rand::thread_rng().gen_range(0.0, 5.0);

        Boid {
            acceleration: Vector2::new(0.0, 0.0),
            velocity: Vector2::new(angle.cos(), angle.cos()),
            position: Vector2::new(x,y),
            r: 2.0,
            max_speed: 3.0,
            max_force: 0.08,
            width: w * 2.0,
            height: h * 2.0
        }
    }

    fn vel_mag(&self) -> f32 {
        self.velocity.magnitude()
    }


    fn pos_x(&self) -> u16 {
        let x = self.position.x.trunc() / 2.0;
        x as u16
    }

    fn pos_y(&self) -> u16 {
        let y = self.position.y.trunc() / 2.0;
        y as u16
    }

    fn run(&mut self, boids: &Vec<Boid>) {
        self.flock(boids);
        self.update();
        self.borders();
        // self.render();
    }

    fn apply_force(&mut self, force: Vector2<f32>) {
        self.acceleration += force;
    }


    fn render(&self) -> &'static str {
        match ( self.velocity.x > 0.0, self.velocity.y < 0.0) {
            (true, false) => "→",
            (true, true) => "↑",
            (false, true) => "←",
            (false, false) => "↓"

                // ↖ ↗ ↘ ↙


        }
    }


    fn flock(&mut self, boids: &Vec<Boid>) {
        let mut sep = self.separate(&boids);
        let mut ali = self.align(&boids);
        let mut coh = self.cohesion(&boids);
        let mut grav = self.gravity();

        // Arbitrarily weight these forces
        sep *= 1.8;
        ali *= 1.0;
        coh *= 1.0;
        grav *= 0.5;

        // Add the force vectors to acceleration
        self.apply_force(sep);
        self.apply_force(ali);
        self.apply_force(coh);
        self.apply_force(grav);
    }

    fn update(&mut self) {
        self.velocity += self.acceleration;

        // println!("accel {:?}", self.acceleration);
        // Limit speed

        self.velocity = {
            if self.velocity.magnitude() > self.max_speed {
                self.velocity.normalize_to(self.max_speed)
            } else {
                self.velocity
            }
        };

        // println!("vel {:?}", self.velocity);

        // println!("before pos {:?}", self.position);
        self.position += self.velocity;
        // println!("after pos {:?}", self.position);

        // Reset accelertion to 0 each cycle
        self.acceleration *= 0.0;
    }


    pub fn seek(&mut self, target: Vector2<f32>) -> Vector2<f32>{
        let mut desired = target - self.position;  // A vector pointing from the position to the target
        // Scale to maximum speed

        desired = desired.normalize();
        desired *= self.max_speed;

        // Steering = Desired minus Velocity
        let mut steer = desired - self.velocity;

        steer = {
            if steer.magnitude() > self.max_force {
                steer.normalize_to(self.max_force)
            } else {
                steer
            }
        };
        steer
    }

    fn borders(&mut self) {
        if (self.position.x < 2.0) { self.position.x = self.width-2.0};
        if (self.position.y < 2.0) {self.position.y = self.height-2.0};
        if (self.position.x > self.width-2.0) {self.position.x = 2.0};
        if (self.position.y > self.height-2.0) { self.position.y = 2.0 };
    }


    fn separate(&mut self, boids: &Vec<Boid>) -> Vector2<f32> {
        let desired_separation = 5.0;
        let mut steer = Vector2::new(0.0, 0.0);
        let mut count = 0.0;
        // For every boid in the system, check if it's too close
        for other in boids {
            let d = self.position.distance(other.position);
            // If the distance is greater than 0 and less than an arbitrary amount (0 when you are yourself)

            if ((d > 0.0) && (d < desired_separation)) {
                // Calculate vector pointing away from neighbor
                let mut diff = self.position - other.position;
                diff.normalize();
                diff /= d; // Weight by distance
                steer += diff;
                count += 1.0;            // Keep track of how many
            }

        }

        // Average -- divide by how many
        if (count > 0.0) {
            steer /= count;
        }

        // As long as the vector is greater than 0
        if (steer.magnitude() > 0.0) {
            // First two lines of code below could be condensed with new PVector setMag() method
            // Not using this method until Processing.js catches up
            steer = steer.normalize_to(self.max_speed);

            // Implement Reynolds: Steering = Desired - Velocity
            steer -= self.velocity;

            steer = {
                if steer.magnitude() > self.max_force {
                    steer.normalize_to(self.max_force)
                } else {
                    steer
                }
            };
        }

        steer
    }


    fn align(&mut self, boids: &Vec<Boid>) -> Vector2<f32> {
        let neighbor_dist = 20.0;
        let mut sum = Vector2::new(0.0,0.0);
        let mut count = 0.0;
        for other in boids {
            let d = self.position.distance(other.position);
            if ((d > 0.0) && (d < neighbor_dist)) {
                sum += other.velocity;
                count += 1.0;
            }
        }
        if (count > 0.0) {
            sum /= count;
            // First two lines of code below could be condensed with new PVector setMag() method
            // Not using this method until Processing.js catches up
            // sum.setMag(maxspeed);

            // Implement Reynolds: Steering = Desired - Velocity
            sum.normalize();
            sum *= self.max_speed;
            let mut steer = sum - self.velocity;
            steer = {
                if steer.magnitude() > self.max_force {
                    steer.normalize_to(self.max_force)
                } else {
                    steer
                }
            };
            return steer;
        }
        else {
            return Vector2::new(0.0,0.0);
        }
    }

    fn cohesion(&mut self, boids: &Vec<Boid>) -> Vector2<f32> {
        let neighbor_dist = 20.0;
        let mut sum = Vector2::new(0.0, 0.0);   // Start with empty vector to accumulate all positions
        let mut count: f32 = 0.0;
        for other in boids {
            let d = self.position.distance(other.position);
            if ((d > 0.0) && (d < neighbor_dist)) {
                sum += other.position;
                count += 1.0;
            }
        }

        if (count > 0.0) {
            sum /= count;
            self.seek(sum)  // Steer towards the position
        }
        else {
            Vector2::new(0.0, 0.0)
        }
    }


    fn gravity(&mut self) -> Vector2<f32> {
        let x = self.width/2.0;
        let y = self.height/2.0;
        self.seek(Vector2::new(x,y))
    }
}
