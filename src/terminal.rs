use crossterm::cursor::MoveTo;
use crossterm::cursor::MoveToNextLine;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::style::Print;
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;

use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;

const KEYBOARD_BUFFER_SIZE: usize = 1024;
const DISPLAY_BUFFER_SIZE: usize = 1024;

const DISPLAY_LINE_LENGTH: u8 = 40;

const BS: u8 = 0xdf;
const CR: u8 = 0x8d;
const ESC: u8 = 0x9b;

/// The 40 x 24 display
pub struct Display {
    line_len: u8,
    display_in: Receiver<u8>,
}

impl Display {
    pub fn new(display_in: Receiver<u8>) -> anyhow::Result<Self> {
        crossterm::execute!(std::io::stdout(), EnterAlternateScreen)?;
        crossterm::terminal::enable_raw_mode()?;

        Ok(Self {
            line_len: 0,
            display_in,
        })
    }

    fn new_line(&mut self) {
        crossterm::execute!(std::io::stdout(), MoveToNextLine(1)).ok();
        self.line_len = 0;
    }

    fn clear_screen(&mut self) {
        crossterm::execute!(std::io::stdout(), Clear(ClearType::All), MoveTo(0, 0)).ok();
        self.line_len = 0;
    }

    fn print_char(&mut self, c: char) {
        crossterm::execute!(std::io::stdout(), Print(c)).ok();
        self.line_len += 1;
    }

    pub fn run(&mut self) {
        loop {
            if self.line_len == DISPLAY_LINE_LENGTH {
                self.new_line();
            }
            if let Ok(x) = self.display_in.recv() {
                if x & 0b1000_0000 != 0 {
                    match x {
                        0xa0..=0xdf => self.print_char((x & 0b0111_1111) as char),
                        CR => self.new_line(),
                        _ => {}
                    }
                }
            }
        }
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        crossterm::terminal::disable_raw_mode().ok();
        crossterm::execute!(std::io::stdout(), LeaveAlternateScreen).ok();
    }
}

pub struct Keyboard {
    keyboard_out: Sender<u8>,
}

impl Keyboard {
    pub fn new(keyboard_out: Sender<u8>) -> Self {
        Self { keyboard_out }
    }

    pub fn run(&mut self) {
        let Self { keyboard_out } = self;
        loop {
            match crossterm::event::read().ok() {
                Some(Event::Key(KeyEvent {
                    code: KeyCode::Char(mut c),
                    ..
                })) => {
                    if c.is_ascii() {
                        c.make_ascii_uppercase();
                    }

                    let c = c as u8;
                    keyboard_out.send(c | 0b1000_0000).ok();
                }
                Some(Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                })) => {
                    keyboard_out.send(CR).ok();
                }
                Some(Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                })) => {
                    keyboard_out.send(ESC).ok();
                }
                Some(Event::Key(KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                })) => {
                    keyboard_out.send(BS).ok();
                }
                _ => {}
            }
        }
    }
}

pub fn keyboard_ports() -> (Sender<u8>, Receiver<u8>) {
    crossbeam_channel::bounded::<u8>(KEYBOARD_BUFFER_SIZE)
}

pub fn display_ports() -> (Sender<u8>, Receiver<u8>) {
    crossbeam_channel::bounded::<u8>(DISPLAY_BUFFER_SIZE)
}
