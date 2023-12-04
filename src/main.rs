use std::path::Path;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use rand::Rng;
use sdl2::image::{InitFlag, LoadTexture, LoadSurface};
use sdl2::keyboard::Scancode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::surface::Surface;
use sdl2::sys::{SDL_DestroyWindow, SDL_SetWindowPosition};
use sdl2::{self, VideoSubsystem, event};
use sdl2::event::{Event, WindowEvent, DisplayEvent};
use sdl2::video::{Window, WindowPos, FlashOperation};

mod stopwatch;
use stopwatch::Stopwatch;

mod vector2;
use vector2::Vector2;

#[derive(PartialEq)]
enum Immunity {
    None,
    OnFirstFocus,
    Always
}

struct Transform {
    position: Vector2,
    width: u32,
    height: u32
}

impl Transform {
    fn new(x: f64, y: f64, w: u32, h: u32) -> Transform {
        Transform { 
            position: Vector2::new(x, y),
            width: w,
            height: h
        }
    }
}

struct Physics {
    transform: Transform,
    velocity: Vector2,
    acceleration: Vector2
}

impl Physics {
    fn new(transform: Transform, velocity: Vector2, acceleration: Vector2) -> Physics {
        Physics {
            transform,
            velocity,
            acceleration
        }
    }

    fn update(&mut self, dt: f64, bounds: &Bounds) {

        // let dt = dt * 0.1;

        const V_LOSS_BOUNCE_X: f64 = 0.3;
        const V_LOSS_BOUNCE_Y: f64 = 0.3;
        const V_LOSS_SECOND: f64 = 0.01;

        
        let w = self.transform.width;
        let h = self.transform.height;

        let cx = self.transform.position.x;
        let cy = self.transform.position.y;
        
        let vx = self.velocity.x;
        let vy = self.velocity.y;
        
        let ax = self.acceleration.x;
        let ay = self.acceleration.y;
        
        // update values
        let dist_x = vx * dt + 0.5*ax*dt*dt;
        let dist_y = vy * dt + 0.5*ay*dt*dt;

        self.velocity.x = vx + ax * dt;
        self.velocity.y = vy + ay * dt;
        
        let lower_y = (bounds.y) as f64;
        let upper_y = (bounds.y + bounds.h as f32) as f64;

        let lower_x = bounds.x as f64;
        let upper_x = (bounds.x + bounds.w as f32) as f64;

        self.transform.position.x += self.velocity.x;
        self.transform.position.y += self.velocity.y;
        
        // straight up 0 velocity if its not moving enough
        // checks
        if cy < lower_y {
            self.velocity.y = vy.abs() - V_LOSS_BOUNCE_Y * vy.abs();

            let dy = lower_y - cy;

            self.transform.position.y = lower_y + dy;

            if vy.abs() < 0.2 {
                self.velocity.y = 0_f64;
            }
        }
        else if cy + h as f64 > upper_y {
            self.velocity.y = - (vy.abs() - V_LOSS_BOUNCE_Y * vy.abs());

            let dy = cy + h as f64 - upper_y;

            // println!("dy: {}", dy);

            self.transform.position.y = (upper_y - h as f64 - dy);

            if vy.abs() < 0.2 {
                self.velocity.y = 0_f64;
            }
        }
        if cx + w as f64 > upper_x {
            self.velocity.x = - (vx.abs() - V_LOSS_BOUNCE_X * vx.abs());

            let dx = cx + w as f64 - upper_x;

            self.transform.position.x = upper_x - w as f64 - dx;

            if vx.abs() < 0.2 {
                self.velocity.x = 0_f64;
            }
        }
        else if cx < lower_x {
            self.velocity.x = vx.abs() - V_LOSS_BOUNCE_X * vx.abs();

            let dx = lower_x - cx;

            self.transform.position.x = lower_x + dx;

            if vx.abs() < 0.2 {
                self.velocity.x = 0_f64;
            }
        }
        // self.transform.position.x += self.velocity.x;
        // self.transform.position.y += self.velocity.y;
    }
}

struct Pest {
    window: Window,
    physics: Physics,
    immunity: Immunity,
    time_alive: f64
}

impl Pest {
    fn new(video_subsys: &VideoSubsystem, immunity: Immunity, physics: Physics, window_name: &str, icon_path: &'static Path, x: i32, y: i32) -> Result<Pest, String> {
        Ok(
            Pest {
                window: create_window(video_subsys, window_name, x, y, 200, 200, icon_path)?,
                immunity,
                physics,
                time_alive: 0.0
            }
        )
    }

    fn draw_image(mut self, texture_path: &'static Path) -> Result<Pest, String> {
        let mut c = self.window
        .into_canvas()
        .accelerated()
        .build()
        .unwrap();

        let texture_creator = c.texture_creator();

        let texture = texture_creator.load_texture(texture_path)?;

        let query = texture.query();

        c.copy(&texture, None, Rect::new(0, 0, query.width, query.height))
        .expect("couldnt copy texture to screen...");

        c.present();

        self.window = c.into_window();

        Ok(self)
    }

    fn draw_color(mut self, color: Color) -> Result<Pest, String> {
        let mut c = self.window
        .into_canvas()
        .accelerated()
        .build()
        .unwrap();
        
        c.set_draw_color(color);
        c.clear();
        c.present();

        self.window = c.into_window();

        Ok(self)
    }
}

struct Bounds {
    x: f32,
    y: f32,
    w: u32,
    h: u32
}

impl Bounds {
    pub fn new(x: f32, y: f32, w: u32, h: u32) -> Bounds {
        Bounds {
            x,
            y,
            w,
            h
        }
    }
}

enum InitMethod {
    WithColor{color: Color},
    WithTexture{texture_path: &'static Path},
    Blank
}

fn kill_pest(pests: &mut Vec<Pest>, window_id: u32) {
    if let Some(p) = find_pest(pests, window_id) {
        let w = &mut p.window;
        let idx = pests.iter().position(|p| {p.window.id() == window_id}).expect("CANNOT FIND WINDOW ID");
        // when the pest geos out of scope, it's window follows and is destroyed.
        pests.remove(idx);
    }
    else {
        println!("window was none lol");
    }
}

fn update_pests(pests: &mut Vec<Pest>, dt: f64, bounds: &Bounds) {
    // account for expensive window moving time
    let update_start = Stopwatch::new();
    for pest in pests {
        // update dt accordingly
        let dt = dt + update_start.elapsed_seconds();

        pest.physics.update(dt, bounds);
        pest.time_alive += dt;
        let x = pest.physics.transform.position.x;
        let y = pest.physics.transform.position.y;

        pest.window.set_minimum_size(1, 1).unwrap();
        // unsafe because i have cancer
        unsafe {
            SDL_SetWindowPosition(pest.window.raw(), x as i32, y as i32);
        }
    }
}

static GRAVITY_X: f64 = -0.0;
static GRAVITY_Y: f64 = 3.81;
static dev_mode: bool = true;

#[allow(clippy::too_many_lines)]
fn main() -> Result<(), String> {

    
    let mut stopwatch = Stopwatch::new();
    let mut spawn_timer = Stopwatch::new();
    let mut rng = rand::thread_rng();
    let mut flood = false;

    let DOGPATH: &Path = Path::new("resources/dog.png");
    
    std::env::set_var("SDL_VIDEO_X11_NET_WM_BYPASS_COMPOSITOR", "0");
    
    let sdl = sdl2::init()?;
    let _img_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG);
    
    let video = sdl.video()?;
    
    let display_rect = video.display_bounds(0)?;
    let screen_bounds = Bounds::new(10.0, 10.0, display_rect.w as u32 - 20, display_rect.h as u32 - 80);
    
    let mut event_pump = sdl.event_pump()?;

    let mut pests = Vec::<Pest>::new();

    let add_pest = |x: i32, y: i32, init_method: InitMethod, immunity: Immunity, initial_vel: Vector2, pests: &mut Vec<Pest>| -> Result<(), String> {

        // pests.clear();

        let physics = Physics::new(
            Transform::new(x as f64, y as f64, 200, 200),
            initial_vel,
            Vector2::new(GRAVITY_X, GRAVITY_Y)
        );
        
        // LORD SAVE ME
        let mut p = Pest::new(&video, immunity, physics, "HELLO", DOGPATH, x, y)?;

        match init_method {
            InitMethod::WithColor { color } => {
                p = p.draw_color(color)?;
            },

            InitMethod::WithTexture { texture_path } => {
                p = p.draw_image(texture_path)?;
            }
            
            InitMethod::Blank => {

            }
        }
        
        pests.push(p);

      
        Ok(())
    };

    let mut add_pest_random = |init_method: InitMethod, immunity: Immunity, initial_vel: Vector2, pests: &mut Vec<Pest>| -> Result<(), String> {

        // pests.clear();

        let x = rng.gen_range(screen_bounds.x..screen_bounds.x + screen_bounds.w as f32) as i32;
        // make the Y spawn a bit lower
        let y = rng.gen_range(screen_bounds.y..screen_bounds.y + screen_bounds.h as f32 - 200_f32) as i32;

        let physics = Physics::new(
            Transform::new(x as f64, y as f64, 200, 200),
            initial_vel,
            Vector2::new(GRAVITY_X, GRAVITY_Y)
        );
        
        // LORD SAVE ME
        let mut p = Pest::new(&video, immunity, physics, "HELLO", DOGPATH, x, y)?;

        match init_method {
            InitMethod::WithColor { color } => {
                p = p.draw_color(color)?;
            },

            InitMethod::WithTexture { texture_path } => {
                p = p.draw_image(texture_path)?;
            }
            
            InitMethod::Blank => {

            }
        }
        
        pests.push(p);

      
        Ok(())
    };

    // sdl.mouse().capture(true);
    
    add_pest(200, 400, InitMethod::WithTexture { texture_path: DOGPATH }, Immunity::OnFirstFocus, Vector2::new(5.0, 0.0), &mut pests)?;
    // add_pest_random(InitMethod::WithTexture { texture_path: DOGPATH }, Vector2::new(5.0, 0.0), &mut pests)?;                    
    // add_pest_random(InitMethod::WithTexture { texture_path: DOGPATH }, Vector2::new(5.0, 0.0), &mut pests)?;                    

    // deltatime
    let mut dt = 0_f64;

    // let on_click = |p: &mut Pest, window_id: u32| -> Result<(), String> {
    //     if p.immunity == Immunity::OnSpawn {
    //         p.immunity = Immunity::None;
    //     }
    //     if p.immunity == Immunity::OnSpawn {
    //         kill_pest(&mut pests, window_id);  
    //         add_pest_random(InitMethod::WithTexture { texture_path: DOGPATH }, Vector2::new_rand(0.0..10.0), &mut pests)?;
    //         // pests[0].physics.velocity = Vector2::new_rand(0.0..10.0);
    //     }

    //     Ok(())
    // };

    'running: loop {
        stopwatch.reset();

        // random spawning
        // if spawn_timer.elapsed_seconds() > 2.5 {
        //     let x = rng.gen_range(screen_bounds.x..screen_bounds.x + screen_bounds.w as f32) as i32;
        //     let y = rng.gen_range(screen_bounds.y..screen_bounds.y + screen_bounds.h as f32) as i32;
        //     add_pest(x, y, InitMethod::WithTexture { texture_path: DOGPATH }, Vector2::new(rng.gen_range(-6.0..6.0), 0.0), &mut pests)?;
        //     spawn_timer.reset();
        // }

        if pests.is_empty() {
            break 'running;
        }

        if pests[0].time_alive > 5.0 {
            flood = true;
        }

        if flood {
            if spawn_timer.elapsed_seconds() > 0.15 {
                spawn_timer.reset();
                add_pest(display_rect.w / 2, display_rect.h / 2, InitMethod::WithTexture { texture_path: DOGPATH }, Immunity::None, Vector2::new_rand(-10.0..10.0), &mut pests)?;   
            }
        }
        
        let mut pressed_codes: Vec<Scancode> = event_pump.keyboard_state().pressed_scancodes().collect();
        for e in event_pump.poll_iter() {
            #[allow(clippy::single_match)]
            match e {
                Event::MouseButtonDown { x, y, window_id, mouse_btn, .. } => {
                    println!("CLICK CLICK");
                    if mouse_btn == MouseButton::Left {
                        kill_pest(&mut pests, window_id);    
                        add_pest_random(InitMethod::WithTexture { texture_path: DOGPATH }, Immunity::None, Vector2::new_rand(0.0..10.0), &mut pests)?;                    
                        // pests[0].physics.velocity = Vector2::new_rand(0.0..10.0);
                    }
                    else if mouse_btn == MouseButton::Right {
                        panic!();
                    }
                },
                Event::Window { window_id, win_event, .. } => {
                    #[allow(clippy::collapsible_match)]
                    match win_event {
                        WindowEvent::TakeFocus => {
                            if pressed_codes.contains(&Scancode::LAlt) {
                                break;
                            }
                            if let Some(p) = find_pest(&mut pests, window_id) {
                                if p.immunity == Immunity::OnFirstFocus {
                                    p.immunity = Immunity::None;
                                    println!("removed immunity");
                                }
                                else if p.immunity == Immunity::None {
                                    println!("KILLED PEST");
                                    kill_pest(&mut pests, window_id);  
                                    add_pest_random(InitMethod::WithTexture { texture_path: DOGPATH }, Immunity::OnFirstFocus, Vector2::new_rand(0.0..10.0), &mut pests)?;
                                    // pests[0].physics.velocity = Vector2::new_rand(0.0..10.0);
                                }
                            }
                            else {
                                println!("window was none lol");
                            }
                        },
                        // to silence clippy...
                        WindowEvent::Minimized => {

                        }
                        _ => ()
                    }
                }
                Event::Quit { .. } => {
                    if dev_mode {
                        break 'running;
                    }
                },
                _ => {}//println!("YOU'ER RETARDED")
            }
        }

        

        // physics
        update_pests(&mut pests, dt, &screen_bounds);

        sleep(Duration::from_secs_f64(1.0/144.0));
        dt = stopwatch.elapsed_seconds();

        // println!("frame completed in {} seconds.", dt);

        stopwatch.reset();

        // pests[0].window.raise();

    }

    Ok(())
}

fn create_window(video_subsystem: &VideoSubsystem, name: &str, x: i32, y: i32, w: u32, h: u32, icon_path: &'static Path) -> Result<Window, String> {
    let mut window = video_subsystem.window(name, w, h)
    .borderless()
    .position(x, y)
    .always_on_top()
    .minimized()
    .allow_highdpi()
    .build()
    .map_err(|e| e.to_string())?;

    let s = Surface::from_file(icon_path)?;

    window.set_icon(s);

    Ok(window)
}


fn find_pest(windows: &mut Vec<Pest>, id: u32) -> Option<&mut Pest> {
    windows.iter_mut()
    .find(|p| p.window.id() == id)
}
