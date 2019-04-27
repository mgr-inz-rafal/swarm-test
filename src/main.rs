extern crate piston_window;

use piston_window::*;

struct GuiConfig {
    width: u16,
    height: u16,
}

fn create_window(gui: &GuiConfig) -> PistonWindow {
    let size: Size = Size {
        height: f64::from(gui.height),
        width: f64::from(gui.width),
    };
    WindowSettings::new("Hello Piston!", size)
        .exit_on_esc(true)
        .build()
        .unwrap()
}

fn game_loop(mut window: PistonWindow) {
    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {
            clear([1.0; 4], g);
            rectangle(
                [1.0, 0.0, 0.0, 1.0], // red
                [0.0, 0.0, 100.0, 100.0],
                c.transform,
                g,
            );
        });
    }
}

fn main() {
    let gui = GuiConfig {
        width: 800,
        height: 600,
    };

    let window = create_window(&gui);
    game_loop(window);

    println!("Koniec!");
}
