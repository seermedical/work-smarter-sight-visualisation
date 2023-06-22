use std::{
    collections::{HashMap, HashSet, VecDeque},
    time::Duration,
};

use data::DataStreamer;
use nannou::{
    color::rgb_u32,
    draw::primitive::texture,
    noise::{NoiseFn, Perlin},
    prelude::*,
    rand::{thread_rng, Rng},
    text::Font,
};

mod data;

const WINDOW_SIZE: u32 = 900;
const NUM_EVENTS: usize = 20;
const NOISE_SCALE: f64 = 500.0;

// 16 Colour Macintosh II Pallette
const WHITE: u32 = 0xffffff;
const ORANGE: u32 = 0xF18F01;
const RED: u32 = 0xF87060;
const PINK: u32 = 0xFF5D73;
const SEA_GREEN: u32 = 0x03B5AA;
const LIGHT_BLUE: u32 = 0x2DE1FC;
const LIME_GREEN: u32 = 0xC2F970;
const LIGHT_GREY: u32 = 0xb9b9b9;
const MEDIUM_GREY: u32 = 0x868686;
const VIOLET: u32 = 0x564D80;
const DARK_VIOLET: u32 = 0x44344F;

const DEVICE_COLOURS: [u32; 5] = [ORANGE, RED, PINK, SEA_GREEN, LIGHT_BLUE];

struct Model {
    window_id: WindowId,
    data_streamer: DataStreamer,
    font: Font,
    event_list: EventList,
    device_list: HashMap<String, Device>,
    perlin: Perlin,
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
        device_list: HashMap::new(),
        perlin: Perlin::new(),
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    if let Some(json) = model.data_streamer.next().unwrap() {
        if let Some(id) = json["data"]["id"].as_str() {
            // let new_device = );
            let mut device = model
                .device_list
                .entry(id.to_string())
                .or_insert(Device::new(app, id.to_string()));

            device.count += 1;
        }

        model.event_list.add_event(app, json);
    }

    model.event_list.update(app);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(rgb_u32(DARK_VIOLET));

    let xs = app.window_rect().x;
    let ys = app.window_rect().y;
    let step_size = 20;
    for x in ((xs.start as i32)..(xs.end as i32)).step_by(step_size) {
        for y in ((ys.start as i32)..(ys.end as i32)).step_by(step_size) {
            let noise_val = model
                .perlin
                .get([x as f64 / NOISE_SCALE, y as f64 / NOISE_SCALE]);

            draw.x_y(x as f32, y as f32)
                .rect()
                .color(rgb_u32(VIOLET))
                .w_h(
                    noise_val as f32 * step_size as f32,
                    noise_val as f32 * step_size as f32,
                );
        }
    }

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

    for (_, device) in &model.device_list {
        device.draw(app, &draw, model.font.clone());
    }
    draw.finish_remaining_drawings();
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
        let slice_min = if json_string.len() <= 9 { 0 } else { 9 };
        let slice_max = if json_string.len() >= 200 {
            200
        } else {
            json_string.len() - 1
        };
        let slice_size = thread_rng().gen_range(100..slice_max);
        let json_string = json_string[slice_min..slice_size].to_string();

        let x_vel = thread_rng().gen_range(30..100) as f32 / 100.0;
        let mut velocity = vec2(x_vel, 0.0) * 20.0;

        let x = if thread_rng().gen_bool(0.5) {

            app.window_rect().left() - 1500.0
        } else {
            velocity.x *= -1.0;
            app.window_rect().right() + 1500.0
        };

        let ys = app.window_rect().y;
        let centre = vec2(x, thread_rng().gen_range(ys.start..ys.end));

        return Self {
            json,
            json_string,
            centre,
            opacity: 1.0,
            velocity,
        };
    }

    fn step(&mut self, app: &App) {
        self.centre += self.velocity * 100.0 * app.duration.since_prev_update.as_secs_f32();
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
        let font_size = 12.0;

        draw.xy(self.centre)
            .text(&self.json_string)
            .width(app.window_rect().w() + 600.0)
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
            if e.centre.x > app.window_rect().right() + variation
                || e.centre.x < app.window_rect().left() - variation
            {
                self.events.remove(i);
            } else {
                e.step(app);
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

struct Device {
    name: String,
    point: Point2,
    count: u64,
    colour: Rgb8,
}

impl Device {
    fn new(app: &App, name: String) -> Self {
        let mut rng = thread_rng();
        let xs = app.window_rect().x;
        let ys = app.window_rect().y;
        let point = vec2(
            rng.gen_range(xs.start..xs.end),
            rng.gen_range(ys.start..ys.end),
        );

        let colour = rgb_u32(DEVICE_COLOURS[rng.gen_range(0..DEVICE_COLOURS.len())]);
        Self {
            name,
            point,
            count: 0,
            colour,
        }
    }

    fn draw(&self, app: &App, draw: &Draw, font: Font) {
        let size = cycle_value_over_time(
            app.duration.since_start,
            Duration::from_secs(1),
            5.0,
            self.count as f32 * 5.0,
        );

        draw.xy(self.point)
            .ellipse()
            .color(self.colour)
            .w_h(size, size);

        draw.xy(self.point + vec2(120.0, -20.0))
            .text(&self.name)
            .width(200.0)
            .left_justify()
            .font(font.clone())
            .font_size(16)
            .color(self.colour);
    }
}

pub fn cycle_value_over_time(
    current_time: Duration,
    cycle_duration: Duration,
    min_value: f32,
    max_value: f32,
) -> f32 {
    let fraction = (current_time.div_f32(cycle_duration.as_secs_f32()))
        .as_secs_f32()
        .fract();
    let cycled_fraction = (fraction - 0.5).abs();
    return map_range(cycled_fraction, 0.0, 0.5, min_value, max_value);
}
