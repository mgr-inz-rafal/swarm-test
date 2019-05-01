#[macro_use(carrier, slot)]
extern crate swarm;
extern crate piston_window;

use piston_window::*;
use swarm::carrier::Carrier;
use swarm::Slot;

const CARRIER_SIZE: f64 = 30.0;
const SLOT_SIZE: f64 = 50.0;

struct GuiConfig {
    width: u16,
    height: u16,
}

struct WorldState {}

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
    mut game: swarm::Swarm,
    mut world: WorldState,
    logic: &Fn(&mut WorldState),
    paint: &Fn(&mut PistonWindow, &swarm::Swarm, Event),
) {
    while let Some(event) = window.next() {
        logic(&mut world);
        paint(&mut window, &game, event);
        game.tick();
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

fn paint_carriers<G>(c: piston_window::Context, g: &mut G, game: &swarm::Swarm)
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

fn game_painter(wnd: &mut PistonWindow, game: &swarm::Swarm, e: Event) {
    wnd.draw_2d(&e, |c, g| {
        clear([1.0; 4], g);
        paint_carriers(c, g, &game);
        paint_slots(c, g, &game);
    });
}

fn main() {
    let gui = GuiConfig {
        width: 800,
        height: 600,
    };

    let world_state = WorldState {};

    let angle: f64 = 180.0;
    let s = angle.sin();

    let mut game = swarm::new();

    game.add_carrier(carrier!(50.0, 50.0));
    game.add_carrier(carrier!(100.0, 90.0));

    game.add_slot(slot!(200.0, 200.0));
    game.add_slot(slot!(210.0, 300.0));

    let window = create_window(&gui);
    game_loop(window, game, world_state, &game_logic, &game_painter);

    println!("Koniec!");
}
