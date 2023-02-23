use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;

use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use crossterm::style::Print;
use std::io::Write;

struct Manzana {}

const KEYBOARD_BUFFER_SIZE: usize = 1024;
const DISPLAY_BUFFER_SIZE: usize = 1024;

fn main() -> anyhow::Result<()> {
    let (cols, rows) = crossterm::terminal::size()?;
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::SetSize(40, 20),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
    )?;

    crossterm::terminal::enable_raw_mode()?;

    let (kb_tx, kb_rx) = crossbeam_channel::bounded::<u8>(KEYBOARD_BUFFER_SIZE);
    let (dsp_tx, dsp_rx) = crossbeam_channel::bounded::<u8>(DISPLAY_BUFFER_SIZE);

    let manzana_thread = std::thread::spawn(move || loop {
        if let Ok(x) = kb_rx.recv() {
            dsp_tx.send(x).ok();
        }
    });

    let kb_thread = std::thread::spawn(move || loop {
        if let Ok(e) = crossterm::event::read() {
            if let Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                modifiers: _, //KeyModifiers::NONE,
                kind: _,      //KeyEventKind::Release,
                state: _,
            }) = e
            {
                if c.is_ascii() {
                    kb_tx.send(c as u8).ok();
                }
            }
        }
    });

    loop {
        if let Ok(x) = dsp_rx.recv() {
            write!(std::io::stdout(), "{x:02x}").ok();
            std::io::stdout().flush().ok();

            if x == b'x' {
                break;
            }
        }
    }

    crossterm::execute!(std::io::stdout(), crossterm::terminal::SetSize(cols, rows))?;
    crossterm::terminal::disable_raw_mode()?;

    manzana_thread.join().unwrap();
    kb_thread.join().unwrap();

    Ok(())
}
