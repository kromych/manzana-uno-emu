use manzana::Manzana;

use terminal::display_rx_tx;
use terminal::keyboard_rx_tx;
use terminal::Display;
use terminal::Keyboard;

mod manzana;
mod terminal;

fn main() -> anyhow::Result<()> {
    let (keyboard_tx, keyboard_rx) = keyboard_rx_tx();
    let (display_tx, display_rx) = display_rx_tx();

    let mut display = Display::new(display_rx)?;
    let mut keyboard = Keyboard::new(keyboard_tx);
    let mut manzana = Manzana::new(keyboard_rx, display_tx);

    let keyboard_thread = std::thread::spawn(move || keyboard.run());
    let dispaly_thread = std::thread::spawn(move || display.run());

    manzana.run();

    keyboard_thread.join().unwrap();
    dispaly_thread.join().unwrap();

    Ok(())
}
