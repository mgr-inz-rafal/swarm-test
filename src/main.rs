#[macro_use(carrier, slot)]
extern crate swarm;
extern crate piston_window;

use piston_window::{
    clear, ellipse, line, rectangle, text, Event, Glyphs, Graphics, PistonWindow, Size,
    TextureSettings, Transformed, WindowSettings,
};
use std::time::Instant;
use swarm::{Carrier, Slot};

const CARRIER_SIZE: f64 = 30.0;
const SLOT_SIZE: f64 = 50.0;
const TARGET_SIZE: f64 = 10.0;
const SIMULATION_TICKER: u128 = (1000.0 / 60.0) as u128; // 60 FPS
const CURRENT_PAYLOAD_FONT_SIZE: f64 = 24.0;
const TARGET_PAYLOAD_FONT_SIZE: f64 = CURRENT_PAYLOAD_FONT_SIZE / 2.3;

struct LabelHelpers {
    slot_label_x_offsets: [Box<Fn(f64) -> f64>; 2],
    slot_label_y_offsets: [Box<Fn(f64) -> f64>; 2],
    slot_label_sizes: [f64; 2],
}

struct GuiData {
    width: u16,
    height: u16,
    fps_counter: u16,
    fps_current: u16,
    label_helpers: LabelHelpers,
}

struct FontCache {
    glyphs: Glyphs,
}

struct WorldState {
    time_since_last_tick: Instant,
    time_since_last_paint: Instant,
}

fn create_window(gui: &GuiData) -> PistonWindow {
    let size: Size = Size {
        height: f64::from(gui.height),
        width: f64::from(gui.width),
    };
    WindowSettings::new("Magister is testing the Swarm library", size)
        .exit_on_esc(true)
        .build()
        .unwrap()
}

fn game_loop(
    mut window: PistonWindow,
    mut font_cache: &mut FontCache,
    gui: &mut GuiData,
    mut game: swarm::Swarm,
    mut world: WorldState,
    logic: &Fn(&mut WorldState),
    paint: &Fn(&mut PistonWindow, &mut FontCache, &swarm::Swarm, &GuiData, Event),
) {
    while let Some(event) = window.next() {
        logic(&mut world);
        paint(&mut window, &mut font_cache, &game, &gui, event);
        gui.fps_counter += 1;

        let now = Instant::now();
        let tick_duration = now - world.time_since_last_tick;
        if tick_duration.as_millis() >= SIMULATION_TICKER {
            game.tick();
            world.time_since_last_tick = Instant::now();
        }

        let fps_duration = now - world.time_since_last_paint;
        if fps_duration.as_secs() >= 1 {
            gui.fps_current = gui.fps_counter;
            gui.fps_counter = 0;
            world.time_since_last_paint = Instant::now();
        }
    }
}

fn game_logic(world: &mut WorldState) {
    // Mutate world state here
}

macro_rules! paint_objects {
    ( $i_objects: expr, $i_shapefn: ident, $i_ctx: ident, $i_gfx: ident, $e_color: expr, $i_size: expr) => {
        $i_objects.iter().for_each(|&i|
            $i_shapefn($e_color, [
                    i.get_position().x - $i_size / 2.0,
                    i.get_position().y - $i_size / 2.0,
                    $i_size,
                    $i_size,
                ],
                $i_ctx.transform,
                $i_gfx,
            )
        )
    };
}

fn paint_carriers_body<G>(c: piston_window::Context, g: &mut G, game: &swarm::Swarm)
where
    G: piston_window::Graphics,
{
    paint_objects!(
        game.get_carriers(),
        ellipse,
        c,
        g,
        [1.0, 0.0, 0.0, 1.0],
        CARRIER_SIZE
    );
}

fn rotate_point(point: (f64, f64), angle: f64, center: (f64, f64)) -> (f64, f64) {
    (
        angle.cos() * (point.0 - center.0) - angle.sin() * (point.1 - center.1) + center.0,
        angle.sin() * (point.0 - center.0) + angle.cos() * (point.1 - center.1) + center.1,
    )
}

fn paint_carriers_angle<G>(c: piston_window::Context, g: &mut G, game: &swarm::Swarm)
where
    G: piston_window::Graphics,
{
    game.get_carriers().iter().for_each(|&x| {
        let point = x.get_position();
        let point = (point.x, point.y);
        let end = (point.0 + CARRIER_SIZE / 2.0, point.1);
        let end = rotate_point(end, x.get_angle(), point);

        line(
            [0.0, 0.0, 1.0, 1.0],
            2.0,
            [point.0, point.1, end.0, end.1],
            c.transform,
            g,
        );
    });
}

fn paint_carriers_target<G>(c: piston_window::Context, g: &mut G, game: &swarm::Swarm)
where
    G: piston_window::Graphics,
{
    game.get_carriers()
        .iter()
        .for_each(|&x| match x.get_target() {
            Some(target) => {
                let position = x.get_position();
                line(
                    [0.0, 0.0, 0.0, 1.0],
                    0.5,
                    [target.0, target.1, position.x, position.y],
                    c.transform,
                    g,
                );

                ellipse(
                    [0.0, 0.0, 1.0, 1.0],
                    [
                        target.0 - TARGET_SIZE / 2.0,
                        target.1 - TARGET_SIZE / 2.0,
                        TARGET_SIZE,
                        TARGET_SIZE,
                    ],
                    c.transform,
                    g,
                )
            }
            None => {}
        });
}

fn paint_slots_body<G>(c: piston_window::Context, g: &mut G, game: &swarm::Swarm)
where
    G: piston_window::Graphics,
{
    paint_objects!(
        game.get_slots(),
        rectangle,
        c,
        g,
        [0.0, 1.0, 0.0, 1.0],
        SLOT_SIZE
    );
}

fn paint_slots_payloads<G>(
    c: piston_window::Context,
    g: &mut G,
    font_cache: &mut FontCache,
    game: &swarm::Swarm,
    gui: &GuiData,
) where
    G: Graphics<Texture = gfx_texture::Texture<gfx_device_gl::Resources>>,
{
    game.get_slots().iter().for_each(|slot| {
        let mut calc_index = 0;
        slot.get_payloads().iter().for_each(|x| match x {
            Some(payload) => {
                let px = slot.get_position().x;
                let py = slot.get_position().y;
                let transform = c.transform.trans(
                    (gui.label_helpers.slot_label_x_offsets[calc_index])(px),
                    (gui.label_helpers.slot_label_y_offsets[calc_index])(py),
                );
                let to_draw = format!("{}", payload);
                let _ = text::Text::new_color(
                    [0.0, 0.0, 0.0, 1.0],
                    gui.label_helpers.slot_label_sizes[calc_index] as u32,
                )
                .draw(
                    &to_draw,
                    &mut font_cache.glyphs,
                    &c.draw_state,
                    transform,
                    g,
                );
                calc_index += 1;
            }
            None => {}
        });
    })
}

fn paint_stats<G>(c: piston_window::Context, g: &mut G, font_cache: &mut FontCache, gui: &GuiData)
where
    G: Graphics<Texture = gfx_texture::Texture<gfx_device_gl::Resources>>,
{
    let mut stats_position = 20.0;
    let transform = c.transform.trans((gui.width - 100) as f64, stats_position);
    let to_draw = format!("{} fps", gui.fps_current);
    let _ = text::Text::new_color([0.0, 0.0, 0.0, 1.0], 16).draw(
        &to_draw,
        &mut font_cache.glyphs,
        &c.draw_state,
        transform,
        g,
    );
    stats_position += 20.0;
    let transform = c.transform.trans((gui.width - 100) as f64, stats_position);
    let _ = text::Text::new_color([0.0, 0.0, 0.0, 1.0], 16).draw(
        "next stat",
        &mut font_cache.glyphs,
        &c.draw_state,
        transform,
        g,
    );
    // stats_position += 20.0;
}

fn game_painter(
    wnd: &mut PistonWindow,
    mut font_cache: &mut FontCache,
    game: &swarm::Swarm,
    gui: &GuiData,
    e: Event,
) {
    wnd.draw_2d(&e, |c, g| {
        clear([1.0; 4], g);
        paint_carriers_body(c, g, &game);
        paint_carriers_angle(c, g, &game);
        paint_carriers_target(c, g, &game);
        paint_slots_body(c, g, &game);
        paint_slots_payloads(c, g, &mut font_cache, &game, &gui);
        paint_stats(c, g, &mut font_cache, &gui);
    });
}

fn main() {
    let mut gui = GuiData {
        width: 800,
        height: 600,
        fps_counter: 0,
        fps_current: 0,
        label_helpers: LabelHelpers {
            slot_label_x_offsets: [
                { Box::new(|pos: f64| pos - CURRENT_PAYLOAD_FONT_SIZE / 1.2) },
                { Box::new(|pos: f64| pos + SLOT_SIZE / 2.0 - SLOT_SIZE / 4.0) },
            ],

            slot_label_y_offsets: [{ Box::new(|pos: f64| pos) }, {
                Box::new(|pos: f64| pos + SLOT_SIZE / 2.0 - 3.0)
            }],
            slot_label_sizes: [CURRENT_PAYLOAD_FONT_SIZE, TARGET_PAYLOAD_FONT_SIZE],
        },
    };

    let world_state = WorldState {
        time_since_last_tick: Instant::now(),
        time_since_last_paint: Instant::now(),
    };

    let mut game = swarm::new();

    game.add_carrier(carrier!(50.0, 50.0));
    game.add_carrier(carrier!(100.0, 90.0));

    game.add_slot(slot!(200.0, 200.0, Some('B'), Some('A')));
    game.add_slot(slot!(210.0, 300.0, Some('A'), Some('B')));
    game.add_slot(slot!(350.0, 350.0, None, None));

    let window = create_window(&gui);
    let mut font_cache = FontCache {
        glyphs: Glyphs::new(
            "fonts/unispace.ttf",
            window.factory.clone(),
            TextureSettings::new(),
        )
        .unwrap(),
    };
    game_loop(
        window,
        &mut font_cache,
        &mut gui,
        game,
        world_state,
        &game_logic,
        &game_painter,
    );

    println!("Koniec!");
}
