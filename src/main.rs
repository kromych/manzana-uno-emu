mod manzana;
mod terminal;

fn main() -> anyhow::Result<()> {
    let (keyboard_out, keyboard_in) = terminal::keyboard_ports();
    let (display_out, display_in) = terminal::display_ports();

    let mut display = terminal::Display::new(display_in)?;
    let mut keyboard = terminal::Keyboard::new(keyboard_out);

    std::thread::spawn(move || keyboard.run());
    std::thread::spawn(move || display.run());

    let (poweroff_out, poweroff_in) = manzana::poweroff();
    let mut manzana = manzana::Manzana::new(keyboard_in, display_out, poweroff_in, poweroff_out);
    manzana.run()?;

    Ok(())
}
