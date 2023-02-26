use manzana::Manzana;

use terminal::display_ports;
use terminal::keyboard_ports;
use terminal::Display;
use terminal::Keyboard;

mod manzana;
mod terminal;

fn main() -> anyhow::Result<()> {
    let (keyboard_out, keyboard_in) = keyboard_ports();
    let (display_out, display_in) = display_ports();

    let mut display = Display::new(display_in)?;
    let mut keyboard = Keyboard::new(keyboard_out);
    let mut manzana = Manzana::new(keyboard_in, display_out);

    std::thread::spawn(move || keyboard.run());
    std::thread::spawn(move || display.run());

    manzana.run();

    Ok(())
}
