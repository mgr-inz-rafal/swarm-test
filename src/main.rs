#[macro_use(carrier, slot)]
extern crate swarm;
extern crate piston_window;

use piston_window::{
    clear, ellipse, line, rectangle, text, Event, Glyphs, Graphics, PistonWindow, Size,
    TextureSettings, Transformed, WindowSettings,
};
use std::time::{Duration, Instant};
use swarm::carrier::Carrier;
use swarm::Slot;

const CARRIER_SIZE: f64 = 30.0;
const SLOT_SIZE: f64 = 50.0;
const SIMULATION_TICKER: u128 = (1000.0 / 60.0) as u128; // 60 FPS

struct GuiConfig {
    width: u16,
    height: u16,
}

struct WorldState {
    // Statistics
    time_since_last_tick: Instant,
}

fn create_window(gui: &GuiConfig) -> PistonWindow {
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
    gui: &GuiConfig,
    mut game: swarm::Swarm,
    mut world: WorldState,
    logic: &Fn(&mut WorldState),
    paint: &Fn(&mut PistonWindow, &swarm::Swarm, &GuiConfig, Event),
) {
    while let Some(event) = window.next() {
        logic(&mut world);
        paint(&mut window, &game, &gui, event);
        let now = Instant::now();
        let duration = now - world.time_since_last_tick;

        if duration.as_millis() < SIMULATION_TICKER {

        } else {
            println!("Tick!");
            game.tick();
            world.time_since_last_tick = Instant::now();
        }
    }
}

fn game_logic(world: &mut WorldState) {
    // Mutate world state here
}

macro_rules! paint_objects {
    ( $i_objects: expr, $i_shapefn: ident, $i_ctx: ident, $i_gfx: ident, $e_color: expr, $i_size: expr) => {
        $i_objects.iter().for_each(|&i|
            $i_shapefn($e_color,          [
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

fn paint_slots<G>(c: piston_window::Context, g: &mut G, game: &swarm::Swarm)
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

fn paint_stats<G>(
    c: piston_window::Context,
    g: &mut G,
    gui: &GuiConfig,
    factory: piston_window::GfxFactory,
) where
    G: Graphics<Texture = gfx_texture::Texture<gfx_device_gl::Resources>>,
{
    let mut stats_position = 20.0;
    let font = "fonts/unispace.ttf";
    let mut glyphs = Glyphs::new(font, factory, TextureSettings::new()).unwrap();
    let transform = c.transform.trans((gui.width - 100) as f64, stats_position);
    let text = text::Text::new_color([0.0, 0.0, 0.0, 1.0], 16).draw(
        "1000 fps",
        &mut glyphs,
        &c.draw_state,
        transform,
        g,
    );
    stats_position += 20.0;
    let transform = c.transform.trans((gui.width - 100) as f64, stats_position);
    let text = text::Text::new_color([0.0, 0.0, 0.0, 1.0], 16).draw(
        "next stat",
        &mut glyphs,
        &c.draw_state,
        transform,
        g,
    );
    // stats_position += 20.0;
}

fn game_painter(wnd: &mut PistonWindow, game: &swarm::Swarm, gui: &GuiConfig, e: Event) {
    let factory = wnd.factory.clone();
    wnd.draw_2d(&e, |c, g| {
        clear([1.0; 4], g);
        paint_carriers_body(c, g, &game);
        paint_carriers_angle(c, g, &game);
        paint_slots(c, g, &game);
        paint_stats(c, g, &gui, factory);
    });
}

fn main() {
    let gui = GuiConfig {
        width: 800,
        height: 600,
    };

    let world_state = WorldState {
        time_since_last_tick: Instant::now(),
    };

    let mut game = swarm::new();

    game.add_carrier(carrier!(50.0, 50.0));
    game.add_carrier(carrier!(100.0, 90.0));

    game.add_slot(slot!(200.0, 200.0));
    game.add_slot(slot!(210.0, 300.0));

    let window = create_window(&gui);
    game_loop(window, &gui, game, world_state, &game_logic, &game_painter);

    println!("Koniec!");
}
