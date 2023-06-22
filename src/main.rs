use std::collections::VecDeque;

use data::DataStreamer;
use nannou::{
    color::rgb_u32,
    draw::primitive::texture,
    prelude::*,
    rand::{thread_rng, Rng},
    text::Font,
};

mod data;

const WINDOW_SIZE: u32 = 900;
const NUM_EVENTS: usize = 20;

// 16 Colour Macintosh II Pallette
const WHITE: u32 = 0xffffff;
const YELLOW: u32 = 0xffff00;
const ORANGE: u32 = 0xff6500;
const RED: u32 = 0xdc0000;
const PINK: u32 = 0xff0097;
const PURPLE: u32 = 0x360097;
const BLUE: u32 = 0x0000ca;
const LIGHT_BLUE: u32 = 0x0097ff;
const LIME_GREEN: u32 = 0x00a800;
const GREEN: u32 = 0x006500;
const BROWN: u32 = 0x653600;
const LIGHT_BROWN: u32 = 0x976536;
const LIGHT_GREY: u32 = 0xb9b9b9;
const MEDIUM_GREY: u32 = 0x868686;
const DARK_GREY: u32 = 0x454545;
const BLACK: u32 = 0x000000;

struct Model {
    window_id: WindowId,
    data_streamer: DataStreamer,
    font: Font,
    event_list: EventList,
}

// impl Drop for Model {
//     fn drop(&mut self) {
//         self.data_stream_thread.thread().
//     }
// }

fn main() {
    nannou::app(model)
        .update(update)
        .loop_mode(LoopMode::RefreshSync)
        .run();
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .title(app.exe_name().unwrap())
        .size(WINDOW_SIZE, WINDOW_SIZE)
        .view(view)
        .build()
        .unwrap();

    app.set_fullscreen_on_shortcut(true);

    let font_path = app.assets_path().unwrap().join("Monaco_Regular.ttf");
    let font_data = std::fs::read(&font_path).unwrap();
    let font = Font::from_bytes(font_data).unwrap();

    Model {
        window_id,
        data_streamer: DataStreamer::new(),
        event_list: EventList::new(font.clone()),
        font,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    if let Some(json) = model.data_streamer.next().unwrap() {
        
        model.event_list.add_event(app, json)
    }

    model.event_list.update(app);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(rgb_u32(BLACK));

    model.event_list.draw(&draw, app);

    // for e in &model.event_list.events {
    //     draw.x_y(0.0, 0.0)
    //     .text(&e.json.to_string())
    //     .font(model.font.clone())
    //     .font_size(12)
    //     .color(rgb_u32(PINK));
    // }

    let fps = app.fps();

    draw.xy(app.window_rect().bottom_left() + vec2(50.0, 50.0))
        .text(&fps.to_string())
        .font(model.font.clone())
        .font_size(12)
        .color(rgb_u32(PINK));

    draw.to_frame(app, &frame).unwrap();
}

struct Event {
    json: serde_json::Value,
    json_string: String,
    centre: Point2,
    opacity: f32,
    velocity: Vec2,
}

impl Event {
    fn new(app: &App, json: serde_json::Value) -> Self {


        let json_string = json.to_string();
        let slice_max = if json_string.len() >= 150 {
            150
        } else {
            json_string.len() - 1
        };
        let slice_size = thread_rng().gen_range(100..slice_max);
        let json_string = json_string[..slice_size].to_string();

        let y_vel = thread_rng().gen_range(-80..80) as f32 / 100.0;
        let velocity = dbg!(vec2(0.0, y_vel as f32) * 15.0);

        let y = if velocity.y > 0.0 {
            app.window_rect().bottom() - 1500.0
        } else {
            app.window_rect().top() + 1500.0
        };

        let xs = app.window_rect().x;
        let centre = vec2(
            thread_rng().gen_range(xs.start..xs.end),
            y,
        );

        return Self {
            json,
            json_string,
            centre,
            opacity: 1.0,
            velocity,
        };
    }

    fn step(&mut self) {
        self.centre += self.velocity;
    }

    fn draw(&self, app: &App, draw: &Draw, font: &Font, colour: &Rgb<u8>) {
        // let text_string = self.json.to_string();
        // let text = text(&text_string)
        //     .font(font.clone())
        //     .font_size(12)
        //     .build(app.window_rect());

        // draw.xy(self.centre)
        //     .path()
        //     .fill()
        //     // .fill_tolerance(0.1)
        //     .color(*colour)
        //     .events(text.path_events());
        let font_size = 26.0;

        draw.xy(self.centre)
            .text(&self.json_string)
            .width(font_size)
            .wrap_by_character()
            .left_justify()
            .font(font.clone())
            .font_size(font_size as u32)
            .color(*colour);
    }
}

struct EventList {
    colour: Rgb<u8>,
    events: VecDeque<Event>,
    font: Font,
}

impl EventList {
    fn add_event(&mut self, app: &App, event_json: serde_json::Value) {
        if self.events.len() < NUM_EVENTS {
            self.events.push_back(Event::new(app, event_json));
        }
    }

    fn new(font: Font) -> Self {
        Self {
            colour: rgb_u32(LIME_GREEN),
            events: VecDeque::with_capacity(NUM_EVENTS),
            font,
        }
    }

    fn update(&mut self, app: &App) {
        let mut i = 0;
        while i < self.events.len() {
            let e = &mut self.events[i];
            let variation = 1500.0;
            if e.centre.y > app.window_rect().top() + variation
                || e.centre.y < app.window_rect().bottom() - variation
            {
                self.events.remove(i);
            } else {
                e.step();
                i += 1;
            }
        }
    }

    fn draw(&self, draw: &Draw, app: &App) {
        for event in &self.events {
            event.draw(app, draw, &self.font, &self.colour)
        }
    }
}
