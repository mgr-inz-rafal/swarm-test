extern crate piston_window;

use piston_window::*;

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

fn game_painter(wnd: &mut PistonWindow, e: Event, world: &WorldState) {
    wnd.draw_2d(&e, |c, g| {
        clear([1.0; 4], g);
        rectangle(
            [1.0, 0.0, 0.0, 1.0], // red
            [world.x, world.y, 100.0, 100.0],
            c.transform,
            g,
        );
    });
}

fn main() {
    let gui = GuiConfig {
        width: 800,
        height: 600,
    };

    let world_state = WorldState { x: 200.0, y: 100.0 };

    let window = create_window(&gui);
    game_loop(window, world_state, &game_logic, &game_painter);

    println!("Koniec!");
}
