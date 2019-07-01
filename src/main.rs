#[macro_use(make_slot_pit, make_slot_spawner)]
extern crate swarm_it;
extern crate piston_window;

use piston_window::{
    clear, ellipse, line, rectangle, text, Event, Glyphs, Graphics, PistonWindow, Size,
    TextureSettings, Transformed, WindowSettings,
};

use std::fs::File;
use std::io::{BufRead, BufReader, Read, Result};
use std::time::Instant;
use swarm_it::{Carrier, Payload, Slot, SlotKind};

const CARRIER_SIZE: f64 = 30.0;
const SLOT_SIZE: f64 = 50.0;
const TARGET_SIZE: f64 = 10.0;
const SIMULATION_TICKER: u128 = (1000.0 / 30.0) as u128; // 30 FPS
const CURRENT_PAYLOAD_FONT_SIZE: f64 = 24.0;
const TARGET_PAYLOAD_FONT_SIZE: f64 = CURRENT_PAYLOAD_FONT_SIZE / 2.3;
const NULL_SLOT_PAYLOAD_CHAR: char = '^';

type MyGameType = swarm_it::Swarm<char>;

struct LabelHelpers {
    slot_label_x_offsets: [Box<Fn(f64) -> f64>; 2],
    slot_label_y_offsets: [Box<Fn(f64) -> f64>; 2],
    slot_label_sizes: [f64; 2],
    carrier_label_x_offset: Box<Fn(f64) -> f64>,
    carrier_label_y_offset: Box<Fn(f64) -> f64>,
    carrier_label_size: f64,
    carrier_state_x_offset: Box<Fn(f64) -> f64>,
    carrier_state_y_offset: Box<Fn(f64) -> f64>,
    carrier_state_size: f64,
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
    mut game: MyGameType,
    mut world: WorldState,
    logic: &Fn(&mut WorldState),
    paint: &Fn(&mut PistonWindow, &mut FontCache, &MyGameType, &GuiData, Event),
) {
    while let Some(event) = window.next() {
        logic(&mut world);
        paint(&mut window, &mut font_cache, &game, &gui, event);
        gui.fps_counter += 1;

        let now = Instant::now();
        let mut tick_duration = now - world.time_since_last_tick;
        while tick_duration.as_millis() >= SIMULATION_TICKER {
            game.tick();
            tick_duration -= std::time::Duration::from_millis(SIMULATION_TICKER as u64);
        }
        world.time_since_last_tick = Instant::now() - tick_duration;

        let fps_duration = now - world.time_since_last_paint;
        if fps_duration.as_secs() >= 1 {
            gui.fps_current = gui.fps_counter;
            gui.fps_counter = 0;
            world.time_since_last_paint = Instant::now();
        }
    }
}

fn game_logic(_world: &mut WorldState) {
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

fn paint_carriers_body<G>(c: piston_window::Context, g: &mut G, game: &MyGameType)
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

fn paint_carriers_angle<G>(c: piston_window::Context, g: &mut G, game: &MyGameType)
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

fn paint_carriers_target<G>(c: piston_window::Context, g: &mut G, game: &MyGameType)
where
    G: piston_window::Graphics,
{
    game.get_carriers().iter().for_each(|&x| {
        if let Some(target) = x.get_target() {
            {
                let target = game.get_slots()[target].get_position();
                let position = x.get_position();
                line(
                    [0.0, 0.0, 0.0, 1.0],
                    0.5,
                    [target.x, target.y, position.x, position.y],
                    c.transform,
                    g,
                );

                ellipse(
                    [0.0, 0.0, 1.0, 1.0],
                    [
                        target.x - TARGET_SIZE / 2.0,
                        target.y - TARGET_SIZE / 2.0,
                        TARGET_SIZE,
                        TARGET_SIZE,
                    ],
                    c.transform,
                    g,
                )
            }
        }
    });
}

fn paint_slots_body<G>(c: piston_window::Context, g: &mut G, game: &MyGameType)
where
    G: piston_window::Graphics,
{
    game.get_slots().iter().for_each(|&i| {
        rectangle(
            if i.is_pit() {
                [0.4, 0.4, 0.4, 1.0]
            } else if i.is_spawner() {
                [0.8, 0.8, 0.8, 1.0]
            } else if i.is_taken_care_of() {
                [0.0, 1.0, 0.0, 1.0]
            } else {
                [0.0, 0.5, 0.0, 1.0]
            },
            [
                i.get_position().x - SLOT_SIZE / 2.0,
                i.get_position().y - SLOT_SIZE / 2.0,
                SLOT_SIZE,
                SLOT_SIZE,
            ],
            c.transform,
            g,
        )
    })
}

fn carrier_state_to_string(state: swarm_it::State) -> &'static str {
    match state {
        swarm_it::State::IDLE => "Idle",
        swarm_it::State::TARGETING(_) => "Targeting",
        swarm_it::State::MOVING(_) => "Moving",
        swarm_it::State::PICKINGUP(_) => "Picking up",
        swarm_it::State::LOOKINGFORTARGET => "Looking where to drop",
        swarm_it::State::NOTARGET => "Can't find target to drop",
        swarm_it::State::DELIVERING(_) => "Delivering payload",
        swarm_it::State::PUTTINGDOWN(_) => "Putting down",
        swarm_it::State::_DEBUG_ => "Beret",
    }
}

fn paint_carriers_payload<G>(
    c: piston_window::Context,
    g: &mut G,
    font_cache: &mut FontCache,
    game: &MyGameType,
    gui: &GuiData,
) where
    G: Graphics<Texture = gfx_texture::Texture<gfx_device_gl::Resources>>,
{
    game.get_carriers().iter().for_each(|carrier| {
        if let Some(payload) = carrier.get_payload() {
            let px = carrier.get_position().x;
            let py = carrier.get_position().y;
            let transform = c.transform.trans(
                (gui.label_helpers.carrier_label_x_offset)(px),
                (gui.label_helpers.carrier_label_y_offset)(py),
            );
            let to_draw = format!("{}", payload.cargo);
            let _ = text::Text::new_color(
                [0.0, 0.0, 0.0, 1.0],
                gui.label_helpers.carrier_label_size as u32,
            )
            .draw(
                &to_draw,
                &mut font_cache.glyphs,
                &c.draw_state,
                transform,
                g,
            );
        }
    });
}

fn paint_carriers_state<G>(
    c: piston_window::Context,
    g: &mut G,
    font_cache: &mut FontCache,
    game: &MyGameType,
    gui: &GuiData,
) where
    G: Graphics<Texture = gfx_texture::Texture<gfx_device_gl::Resources>>,
{
    game.get_carriers().iter().for_each(|carrier| {
        let px = carrier.get_position().x;
        let py = carrier.get_position().y;
        let transform = c.transform.trans(
            (gui.label_helpers.carrier_state_x_offset)(px),
            (gui.label_helpers.carrier_state_y_offset)(py),
        );
        let to_draw = carrier_state_to_string(carrier.get_state());
        let _ = text::Text::new_color(
            [0.0, 0.0, 0.0, 1.0],
            gui.label_helpers.carrier_state_size as u32,
        )
        .draw(
            &to_draw,
            &mut font_cache.glyphs,
            &c.draw_state,
            transform,
            g,
        );
    });
}

fn paint_slots_payloads<G>(
    c: piston_window::Context,
    g: &mut G,
    font_cache: &mut FontCache,
    game: &MyGameType,
    gui: &GuiData,
) where
    G: Graphics<Texture = gfx_texture::Texture<gfx_device_gl::Resources>>,
{
    game.get_slots().iter().for_each(|slot| {
        let mut calc_index = 0;
        slot.get_payloads().iter().for_each(|x| {
            if let Some(payload) = x {
                let px = slot.get_position().x;
                let py = slot.get_position().y;
                let transform = c.transform.trans(
                    gui.label_helpers.slot_label_x_offsets[calc_index](px),
                    gui.label_helpers.slot_label_y_offsets[calc_index](py),
                );
                let to_draw = format!("{}", payload.cargo);
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
            }
            calc_index += 1;
        });
    })
}

fn paint_stats<G>(c: piston_window::Context, g: &mut G, font_cache: &mut FontCache, gui: &GuiData)
where
    G: Graphics<Texture = gfx_texture::Texture<gfx_device_gl::Resources>>,
{
    let mut stats_position = 20.0;
    let transform = c
        .transform
        .trans(f64::from(gui.width - 100), stats_position);
    let to_draw = format!("{} fps", gui.fps_current);
    let _ = text::Text::new_color([0.0, 0.0, 0.0, 1.0], 16).draw(
        &to_draw,
        &mut font_cache.glyphs,
        &c.draw_state,
        transform,
        g,
    );
    stats_position += 20.0;
    let transform = c
        .transform
        .trans(f64::from(gui.width - 100), stats_position);
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
    game: &MyGameType,
    gui: &GuiData,
    e: Event,
) {
    wnd.draw_2d(&e, |c, g| {
        clear([1.0; 4], g);
        paint_slots_body(c, g, &game);
        paint_slots_payloads(c, g, &mut font_cache, &game, &gui);
        paint_stats(c, g, &mut font_cache, &gui);
        paint_carriers_body(c, g, &game);
        paint_carriers_angle(c, g, &game);
        paint_carriers_target(c, g, &game);
        paint_carriers_payload(c, g, &mut font_cache, &game, &gui);
        paint_carriers_state(c, g, &mut font_cache, &game, &gui);
    });
}

fn load_slots_from_file(file: &str, game: &mut MyGameType) -> Result<()> {
    let file = File::open(file)?;
    let mut buffer = BufReader::new(file);

    let _ = buffer.by_ref().lines().next().unwrap().unwrap();
    let height = buffer.by_ref().lines().next().unwrap().unwrap();
    let _ = buffer.by_ref().lines().next().unwrap().unwrap(); // Separator

    let mut source = Vec::new();
    for line in buffer
        .by_ref()
        .lines()
        .take(height.parse::<usize>().unwrap())
    {
        source.push(line);
    }

    let _ = buffer.by_ref().lines().next().unwrap().unwrap(); // Separator

    let mut target = Vec::new();
    for line in buffer
        .by_ref()
        .lines()
        .take(height.parse::<usize>().unwrap())
    {
        target.push(line);
    }

    for (si, sv) in source.iter().enumerate() {
        let source_chars: Vec<char> = sv.as_ref().unwrap().chars().collect();
        let target_chars: Vec<char> = target[si].as_ref().unwrap().chars().collect();
        for (ti, tv) in source_chars.iter().enumerate() {
            let source_payload = if *tv == NULL_SLOT_PAYLOAD_CHAR {
                None
            } else {
                Some(Payload::new(*tv))
            };
            let target_payload = if target_chars[ti] == NULL_SLOT_PAYLOAD_CHAR {
                None
            } else {
                Some(Payload::new(target_chars[ti]))
            };
            game.add_slot(Slot::new(
                SLOT_SIZE as f64 * 2.0 + ti as f64 * (SLOT_SIZE as f64 * 1.1),
                SLOT_SIZE as f64 * 2.0 + si as f64 * (SLOT_SIZE as f64 * 1.1),
                source_payload,
                target_payload,
                swarm_it::SlotKind::CLASSIC,
            ));
        }
    }

    Ok(())
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
            carrier_label_x_offset: { Box::new(|pos: f64| pos + CARRIER_SIZE / 2.0) },
            carrier_label_y_offset: { Box::new(|pos: f64| pos) },
            carrier_label_size: 16.0,
            carrier_state_x_offset: { Box::new(|pos: f64| pos - CARRIER_SIZE / 2.0) },
            carrier_state_y_offset: { Box::new(|pos: f64| pos + CARRIER_SIZE / 1.2) },
            carrier_state_size: 12.0,
        },
    };

    let world_state = WorldState {
        time_since_last_tick: Instant::now(),
        time_since_last_paint: Instant::now(),
    };

    let mut game = swarm_it::Swarm::new();

    game.add_carrier(Carrier::new(50.0, 50.0));

    game.add_slot(Slot::new(
        200.0,
        200.0,
        Some(Payload::new('A')),
        None,
        SlotKind::CLASSIC,
    ));
    game.add_slot(Slot::new(
        300.0,
        250.0,
        Some(Payload::new('A')),
        None,
        SlotKind::CLASSIC,
    ));
    game.add_slot(Slot::new(
        400.0,
        300.0,
        Some(Payload::new('A')),
        None,
        SlotKind::CLASSIC,
    ));
    game.add_slot(Slot::new(300.0, 350.0, None, None, SlotKind::CLASSIC));

    /*
    if let Err(e) = load_slots_from_file("test_layouts/test04.txt", &mut game) {
        panic!(e.to_string());
    }
    */

    game.add_slot(make_slot_pit!(0.0, 0.0));
    game.add_slot(make_slot_pit!(500.0, 500.0));
    //    game.add_slot(make_slot_spawner!(100.0, 0.0));

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
