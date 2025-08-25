#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::default;
use std::process::exit;

use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const PLAYER_WIDTH: i32 = 32;
const PLAYER_HEIGHT: i32 = 5;

#[derive(Default, Clone, Copy)]
struct Vector {
    x: f32,
    y: f32,
    w: f32,
}

impl Vector {
    fn dot(&self, other: Vector) -> f32 {
        self.x * other.x + self.y * other.y
    }

    fn reflection(&self, normal: Vector) -> Vector {
        let angle = self.dot(normal);
        let x = self.x - (2.0 * angle) * normal.x;
        let y = self.y - (2.0 * angle) * normal.y;

        Vector {
            x,
            y,
            ..Default::default()
        }
    }
}
#[derive(Default, Clone, Copy)]
struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}
impl Rect {
    fn overlaps(&self, other: Rect) -> bool {
        let horizontal = self.x <= other.x + other.width && self.x + self.width >= other.x;
        let vertical = self.y <= other.y + other.height && self.y + self.height >= other.y;
        vertical && horizontal
    }
}

struct Player {
    rect: Rect,
    velocity: i32,
}
struct Ball {
    rect: Rect,
    velocity: Vector,
}
/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    p1: Player,
    ball: Ball,
    p2: Player,
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Pedals")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new();

    let res = event_loop.run(|event, elwt| {
        // Draw the current frame
        if let Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } = event
        {
            world.draw(pixels.frame_mut());
            if let Err(err) = pixels.render() {
                log_error("pixels.render", err);
                elwt.exit();
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                elwt.exit();
                return;
            }

            #[allow(clippy::collapsible_if)]
            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    elwt.exit();
                    return;
                }
            }
            if input.key_held(KeyCode::ArrowLeft) {
                world.p1.velocity = -2;
            } else if input.key_held(KeyCode::ArrowRight) {
                world.p1.velocity = 2;
            } else {
                world.p1.velocity = 0;
            }
            // Update internal state and request a redraw
            world.update();
            window.request_redraw();
        }
    });
    res.map_err(|e| Error::UserDefined(Box::new(e)))
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        Self {
            p1: Player {
                rect: Rect {
                    x: WIDTH as i32 / 2,
                    y: HEIGHT as i32 - PLAYER_HEIGHT - 1,
                    width: PLAYER_WIDTH,
                    height: PLAYER_HEIGHT,
                },
                velocity: 0,
            },
            ball: Ball {
                rect: Rect {
                    x: WIDTH as i32 / 2,
                    y: HEIGHT as i32 / 2,
                    width: 5,
                    height: 5,
                },
                velocity: Vector {
                    x: 1.0,
                    y: 1.0,
                    ..Default::default()
                },
            },
            p2: Player {
                rect: Rect {
                    x: WIDTH as i32 / 2,
                    y: 1,
                    width: PLAYER_WIDTH,
                    height: PLAYER_HEIGHT,
                },
                velocity: 2,
            },
        }
    }

    fn update_player(p: &mut Player) {
        let mut newp = p.rect.x + p.velocity;
        if newp > 0 && newp + p.rect.width < WIDTH as i32 {
            p.rect.x = newp;
        }
    }

    fn update(&mut self) {
        let center_p2 = self.p2.rect.x + self.p2.rect.width / 2;
        if self.ball.velocity.x > 0.0 && self.p2.velocity < 0 {
            self.p2.velocity = 1;
        } else if self.ball.velocity.x < 0.0 && self.p2.velocity > 0 {
            self.p2.velocity = -1;
        }
        World::update_player(&mut self.p2);
        World::update_player(&mut self.p1);
        self.ball.rect.x += self.ball.velocity.x.round() as i32;
        self.ball.rect.y += self.ball.velocity.y.round() as i32;

        let mut ball_v = self.ball.velocity;

        if self.ball.rect.overlaps(self.p2.rect) {
            ball_v = self.ball.velocity.reflection(Vector {
                x: 0.0,
                y: 1.0,
                w: 0.0,
            });
        } else if self.ball.rect.overlaps(self.p1.rect) {
            ball_v = self.ball.velocity.reflection(Vector {
                x: 0.0,
                y: -1.0,
                w: 0.0,
            });
        } else if self.ball.rect.x <= 0 {
            ball_v = self.ball.velocity.reflection(Vector {
                x: 1.0,
                y: 0.0,
                w: 0.0,
            });
        } else if self.ball.rect.x + self.ball.rect.width >= WIDTH as i32 {
            ball_v = self.ball.velocity.reflection(Vector {
                x: -1.0,
                y: 0.0,
                w: 0.0,
            });
        }
        self.ball.velocity = ball_v;
    }
    fn draw_rect(&self, frame: &mut [u8], rect: Rect, color: [u8; 4]) {
        for i in rect.y..rect.y + rect.height {
            if i >= HEIGHT as i32 || i < 0 {
                continue;
            }
            for j in rect.x..rect.x + rect.width {
                if j >= WIDTH as i32 || j < 0 {
                    continue;
                }

                let index = (j + i * WIDTH as i32) as usize * 4;

                frame[index..index + 4].copy_from_slice(&color);
            }
        }
    }

    fn draw(&self, frame: &mut [u8]) {
        frame.fill(0x00);
        self.draw_rect(frame, self.p1.rect, [0xff, 0, 0, 0xff]);
        self.draw_rect(frame, self.p2.rect, [0x0, 0xff, 0, 0xff]);
        self.draw_rect(frame, self.ball.rect, [0xff, 0xff, 0xff, 0xff]);
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
