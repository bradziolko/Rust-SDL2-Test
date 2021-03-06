﻿#[macro_use]
mod events;
pub mod data;
pub mod gfx;

use self::gfx::Sprite;
use sdl2::render::{Renderer, BlendMode};
use sdl2::pixels::Color;
use sdl2_ttf::Sdl2TtfContext;
use std::collections::HashMap;
use std::path::Path;

struct_events! {
    keyboard: {
        key_escape: Escape,
        key_up: Up,
        key_down: Down,
        key_left: Left,
        key_right: Right,
        key_space: Space,
        key_enter: Enter
    },
    else: {
        quit: Quit { .. }
    }
}

pub struct Phi<'a> {
    pub events: Events,
    pub renderer: Renderer<'a>,
    pub ttf_context: &'a Sdl2TtfContext,

    cached_fonts: HashMap<(&'static str, i32), ::sdl2_ttf::Font<'a>>
}

impl<'a> Phi<'a> {
    fn new(events: Events, renderer: Renderer<'a>, ttf_context: &'a Sdl2TtfContext) -> Phi<'a> {
        Phi {
            events: events,
            renderer: renderer,
            ttf_context: ttf_context,//::sdl2_ttf::init().unwrap(),
            cached_fonts: HashMap::new()
        }
    }

    pub fn output_size(&self) -> (f64, f64) {
        let (w, h) = self.renderer.output_size().unwrap();
        (w as f64, h as f64)
    }

    pub fn ttf_str_sprite(&mut self, text: &str, font_path: &'static str, size: i32, color: Color) -> Option<Sprite> {
        if let Some(font) = self.cached_fonts.get(&(font_path, size)) {
            return font.render(text).blended(color).ok()
                .and_then(|surface| self.renderer.create_texture_from_surface(&surface).ok())
                .map(Sprite::new)
        }
        //::sdl2_ttf::Font::from_file(Path::new(font_path), size).ok()
        self.ttf_context.load_font(Path::new(font_path), size as u16).ok()
            .and_then(|font| {
                self.cached_fonts.insert((font_path, size), font);
                self.ttf_str_sprite(text, font_path, size, color)
        })
    }
}

pub enum ViewAction {
    None,
    Quit,
    ChangeView(Box<View>)
}

pub trait View {
    fn render(&mut self, context: &mut Phi, elapsed: f64) -> ViewAction;
}

pub fn spawn<F>(title: &str, init: F)
where F: Fn(&mut Phi) -> Box<View> {
    // Initialize SDL2
    let sdl_context = ::sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let mut timer = sdl_context.timer().unwrap();
    let _image_context = ::sdl2_image::init(::sdl2_image::INIT_PNG).unwrap();
    let _ttf_context = ::sdl2_ttf::init().unwrap();

    // Create the window
    let window = video.window(title, 800, 600)
        .position_centered().opengl().resizable()
        .build().unwrap();

    let mut context = Phi::new(
        Events::new(sdl_context.event_pump().unwrap()),
        window.renderer()
            .accelerated()
            .build().unwrap(),
        &_ttf_context);

    let mut current_view = init(&mut context);

    //Frame Timing
    let interval = 1_000 / 60;
    let mut before = timer.ticks();
    let mut last_second = timer.ticks();
    let mut fps = 0 as u16;

    loop {
        //Frame Timing
        let now = timer.ticks();
        let dt = now - before;
        let elapsed = dt as f64 / 1_000.0;

        if dt < interval {
            timer.delay(interval - dt);
            continue;
        }

        before = now;
        fps += 1;

        if now - last_second > 1_000 {
            println!("FPS: {}", fps);
            last_second = now;
            fps = 0;
        }

        //Logic and Rendering
        context.events.pump(&mut context.renderer);

        match current_view.render(&mut context, elapsed) {
            ViewAction::None => context.renderer.present(),
            ViewAction::Quit => break,

            ViewAction::ChangeView(new_view) => current_view = new_view
        }
    }
}