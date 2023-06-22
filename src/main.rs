use std::collections::VecDeque;

use data::DataStreamer;
use nannou::{color::rgb_u32, prelude::*, text::Font};

mod data;

const WINDOW_SIZE: u32 = 900;
const NUM_DATA: usize = 20;

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
    data_points: VecDeque<serde_json::Value>,
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

    let font_path = app.assets_path().unwrap().join("FiraCode-Regular.ttf");
    let font_data = std::fs::read(&font_path).unwrap();
    let font = Font::from_bytes(font_data).unwrap();

    Model {
        window_id,
        data_streamer: DataStreamer::new(),
        font,
        data_points: VecDeque::with_capacity(NUM_DATA),
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    if let Some(json) = model.data_streamer.next().unwrap() {
        model.data_points.push_back(json);

        if model.data_points.len() > NUM_DATA {
            model.data_points.pop_front();
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(rgb_u32(BLACK));

    for (i, data) in model.data_points.iter().enumerate() {
        let y = map_range(i, 0, 19, 0.0, app.window_rect().bottom());
        draw.x_y(0.0, y)
            .text(&data.to_string())
            .font(model.font.clone())
            .color(rgb_u32(LIME_GREEN));
    }

    draw.to_frame(app, &frame).unwrap();
}

struct Event {
    json: serde_json::Value,
    centre: Point2,
    colour: Rgb<u8>,
    opacity: f32,
}

impl Event {
    fn new(json: serde_json::Value, centre: Point2) -> Self {
        return Self {
            json,
            centre,
            colour: rgb_u32(LIME_GREEN),
            opacity: 1.0,
        };
    }
}

struct EventList {
    events: Vec<Event>,
}

impl EventList {
    fn add_event(&mut self, event_json: serde_json::Value, start: Point2) {
        self.events.push(Event::new(event_json, start))
    }
}
