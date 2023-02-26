mod manzana;
mod terminal;

fn main() -> anyhow::Result<()> {
    let (keyboard_out, keyboard_in) = terminal::keyboard_ports();
    let (display_out, display_in) = terminal::display_ports();

    let mut display = terminal::Display::new(display_in)?;
    let mut keyboard = terminal::Keyboard::new(keyboard_out);
    let mut manzana = manzana::Manzana::new(keyboard_in, display_out);

    std::thread::spawn(move || keyboard.run());
    std::thread::spawn(move || display.run());

    manzana.run();

    Ok(())
}
