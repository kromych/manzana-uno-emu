use crossterm::cursor::MoveTo;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;

use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;

use crossterm::cursor::MoveToNextLine;
use crossterm::style::Print;

/*
KBD             .EQ     $D010           PIA.A keyboard input
KBDCR           .EQ     $D011           PIA.A keyboard control register
DSP             .EQ     $D012           PIA.B display output register
DSPCR           .EQ     $D013           PIA.B display control register

; KBD b7..b0 are inputs, b6..b0 is ASCII input, b7 is constant high
;     Programmed to respond to low to high KBD strobe
; DSP b6..b0 are outputs, b7 is input
;     CB2 goes low when data is written, returns high when CB1 goes high
; Interrupts are enabled, though not used. KBD can be jumpered to IRQ,
; whereas DSP can be jumpered to NMI.

*/

fn to_apple1_char_code(c: char) -> Tecla {
    let mut c = c;
    if c.is_ascii() {
        c.make_ascii_uppercase();
    }

    let code = match c as u8 {
        0x00..=0x1f => '@' as u8,
        0x20..=0x3f => c as u8,
        0x40..=0x5f => (c as u8 - 0x40) as u8,
        0x60..=0xff => '_' as u8,
    };

    Tecla::Char(code)
}

fn from_apple1_key(t: Tecla) -> char {
    match t {
        Tecla::Char(c) => match c {
            0x20..=0x3f => char::from_u32(c as u32).unwrap_or('@'),
            0x00..=0x1f => char::from_u32((c + 0x40) as u32).unwrap_or('@'),
            _ => '@',
        },
        Tecla::Enter | Tecla::Esc | Tecla::ClearScreen | Tecla::Reset | Tecla::PowerOff => '@',
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Tecla {
    /// Symbol key with the Apple 1 character code:
    ///
    ///     0 	1 	2 	3 	4 	5 	6 	7 	8 	9 	A 	B 	C 	D 	E 	F
    /// -----------------------------------------------------------------
    /// 0x  @ 	A 	B 	C 	D 	E 	F 	G 	H 	I 	J 	K 	L 	M 	N 	O
    /// 1x  P 	Q 	R 	S 	T 	U 	V 	W 	X 	Y 	Z 	[ 	\ 	] 	^ 	_
    /// 2x   	! 	" 	# 	$ 	% 	& 	' 	( 	) 	* 	+ 	, 	- 	. 	/
    /// 3x  0 	1 	2 	3 	4 	5 	6 	7 	8 	9 	: 	; 	< 	= 	> 	?
    Char(u8),
    /// The Enter key
    Enter,
    /// The Escape key
    Esc,
    /// The Clear Screen key, 'PgDn'
    ClearScreen,
    /// The Reset key, 'Home'
    Reset,
    /// The Power Off key, 'End'
    PowerOff,
}

const KEYBOARD_BUFFER_SIZE: usize = 1024;
const DISPLAY_BUFFER_SIZE: usize = 1024;

const DISPLAY_LINE_LENGTH: u8 = 40;

/// The 40 x 24 display
pub struct Display {
    line_len: u8,
    display_in: Receiver<Tecla>,
}

impl Display {
    pub fn new(display_in: Receiver<Tecla>) -> anyhow::Result<Self> {
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

    fn print_char(&mut self, x: Tecla) {
        crossterm::execute!(std::io::stdout(), Print(from_apple1_key(x))).ok();
        self.line_len += 1;
    }

    pub fn run(&mut self) {
        loop {
            if self.line_len == DISPLAY_LINE_LENGTH {
                self.new_line();
            }
            if let Ok(x) = self.display_in.recv() {
                match x {
                    Tecla::Char(_) => self.print_char(x),
                    Tecla::Enter => self.new_line(),
                    Tecla::Esc => {}
                    Tecla::PowerOff => {
                        break;
                    }
                    Tecla::ClearScreen | Tecla::Reset => {
                        self.clear_screen();
                    }
                }
            }
        }
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        crossterm::execute!(std::io::stdout(), LeaveAlternateScreen).ok();
        crossterm::terminal::disable_raw_mode().ok();
    }
}

pub struct Keyboard {
    keyboard_out: Sender<Tecla>,
}

impl Keyboard {
    pub fn new(keyboard_out: Sender<Tecla>) -> Self {
        Self { keyboard_out }
    }

    pub fn run(&mut self) {
        let Self { keyboard_out } = self;
        loop {
            match crossterm::event::read().ok() {
                Some(Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                })) => {
                    keyboard_out.send(to_apple1_char_code(c)).ok();
                }
                Some(Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                })) => {
                    keyboard_out.send(Tecla::Enter).ok();
                }
                Some(Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                })) => {
                    keyboard_out.send(Tecla::Esc).ok();
                }
                Some(Event::Key(KeyEvent {
                    code: KeyCode::PageDown,
                    ..
                })) => {
                    keyboard_out.send(Tecla::ClearScreen).ok();
                }
                Some(Event::Key(KeyEvent {
                    code: KeyCode::Home,
                    ..
                })) => {
                    keyboard_out.send(Tecla::Reset).ok();
                    break;
                }
                Some(Event::Key(KeyEvent {
                    code: KeyCode::End, ..
                })) => {
                    keyboard_out.send(Tecla::PowerOff).ok();
                    break;
                }
                _ => {}
            }
        }
    }
}

pub fn keyboard_ports() -> (Sender<Tecla>, Receiver<Tecla>) {
    crossbeam_channel::bounded::<Tecla>(KEYBOARD_BUFFER_SIZE)
}

pub fn display_ports() -> (Sender<Tecla>, Receiver<Tecla>) {
    crossbeam_channel::bounded::<Tecla>(DISPLAY_BUFFER_SIZE)
}
