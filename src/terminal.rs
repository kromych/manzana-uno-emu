use crossterm::cursor::MoveToNextLine;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::style::Print;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::ScrollUp;

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
    line_no: u16,
    port_in: Receiver<Tecla>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tecla {
    Char(u8),
    PowerOff,
}

impl Display {
    pub fn new(port_in: Receiver<Tecla>) -> anyhow::Result<Self> {
        crossterm::execute!(std::io::stdout(), EnterAlternateScreen)?;
        crossterm::terminal::enable_raw_mode()?;

        Ok(Self {
            line_len: 0,
            line_no: 0,
            port_in,
        })
    }

    fn new_line(&mut self) {
        crossterm::execute!(std::io::stdout(), MoveToNextLine(1)).ok();
        self.line_len = 0;
        self.line_no += 1;
        if let Ok((_cols, rows)) = crossterm::terminal::size() {
            if self.line_no >= rows {
                crossterm::execute!(std::io::stdout(), ScrollUp(1)).ok();
            }
        }
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
            if let Ok(x) = self.port_in.recv() {
                match x {
                    Tecla::Char(x) => {
                        let x = x & 0b0111_1111;
                        match x {
                            0x20..=0x6f => self.print_char(x as char),
                            0x0d => self.new_line(),
                            _ => {}
                        }
                    }
                    Tecla::PowerOff => break,
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

pub fn display_ports() -> (Sender<Tecla>, Receiver<Tecla>) {
    crossbeam_channel::bounded(DISPLAY_BUFFER_SIZE)
}

pub struct Keyboard {
    port_out: Sender<Tecla>,
}

impl Keyboard {
    pub fn new(port_out: Sender<Tecla>) -> Self {
        Self { port_out }
    }

    pub fn run(&mut self) {
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
                    self.port_out.send(Tecla::Char(c | 0b1000_0000)).ok();
                }
                Some(Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                })) => {
                    self.port_out.send(Tecla::Char(CR)).ok();
                }
                Some(Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                })) => {
                    self.port_out.send(Tecla::Char(ESC)).ok();
                }
                Some(Event::Key(KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                })) => {
                    self.port_out.send(Tecla::Char(BS)).ok();
                }
                Some(Event::Key(KeyEvent {
                    code: KeyCode::End, ..
                })) => {
                    self.port_out.send(Tecla::PowerOff).ok();
                    break;
                }
                _ => {}
            }
        }
    }
}

pub fn keyboard_ports() -> (Sender<Tecla>, Receiver<Tecla>) {
    crossbeam_channel::bounded(KEYBOARD_BUFFER_SIZE)
}
