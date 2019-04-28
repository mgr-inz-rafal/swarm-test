extern crate gauchos;
extern crate piston_window;

use piston_window::*;

const GAUCHO_SIZE: f64 = 30.0;

struct GuiConfig {
    width: u16,
    height: u16,
}

struct WorldState {
    x: f64,
    y: f64,
}

fn create_window(gui: &GuiConfig) -> PistonWindow {
    let size: Size = Size {
        height: f64::from(gui.height),
        width: f64::from(gui.width),
    };
    WindowSettings::new("Magister is testing the Gauchos library", size)
        .exit_on_esc(true)
        .build()
        .unwrap()
}

fn game_loop(
    mut window: PistonWindow,
    mut world: WorldState,
    logic: &Fn(&mut WorldState),
    paint: &Fn(&mut PistonWindow, Event, &WorldState),
) {
    while let Some(event) = window.next() {
        logic(&mut world);
        paint(&mut window, event, &world)
    }
}

fn game_logic(world: &mut WorldState) {
    world.x = world.x + 0.1;
}

fn paint_gauchos<G>(c: piston_window::Context, g: &mut G)
where
    G: piston_window::Graphics,
{
    let indices = gauchos::get_active_gauchos_indices();
    indices
        .iter()
        .for_each(|&gi| match gauchos::get_gaucho_position(gi) {
            Ok(pos) => ellipse(
                [1.0, 0.0, 0.0, 1.0],
                [
                    pos[0] - GAUCHO_SIZE / 2.0,
                    pos[1] - GAUCHO_SIZE / 2.0,
                    GAUCHO_SIZE,
                    GAUCHO_SIZE,
                ],
                c.transform,
                g,
            ),
            Err(msg) => (),
        })
}

fn game_painter(wnd: &mut PistonWindow, e: Event, world: &WorldState) {
    wnd.draw_2d(&e, |c, g| {
        clear([1.0; 4], g);
        paint_gauchos(c, g);
    });
}

fn main() {
    let gui = GuiConfig {
        width: 800,
        height: 600,
    };

    let world_state = WorldState { x: 200.0, y: 100.0 };

    let i = gauchos::add_gaucho();
    let _ = gauchos::set_gaucho_position(i.unwrap(), [0.0, 0.0]);

    let i = gauchos::add_gaucho();
    let _ = gauchos::set_gaucho_position(i.unwrap(), [300.0, 150.0]);

    let window = create_window(&gui);
    game_loop(window, world_state, &game_logic, &game_painter);

    println!("Koniec!");
}
