use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;

use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;

use std::io::Write;

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

/*
   The computer used a Signetics 2513 64×8×5 Character Generator, capable of displaying uppercase characters,
   numbers and basic punctuation and math symbols with a 5x8 pixel font

       0 	1 	2 	3 	4 	5 	6 	7 	8 	9 	A 	B 	C 	D 	E 	F
   0x 	@ 	A 	B 	C 	D 	E 	F 	G 	H 	I 	J 	K 	L 	M 	N 	O
   1x 	P 	Q 	R 	S 	T 	U 	V 	W 	X 	Y 	Z 	[ 	\ 	] 	^ 	_
   2x 		! 	" 	# 	$ 	% 	& 	' 	( 	) 	* 	+ 	, 	- 	. 	/
   3x 	0 	1 	2 	3 	4 	5 	6 	7 	8 	9 	: 	; 	< 	= 	> 	?

*/

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Tecla {
    Printable(u8),
    Enter,
    Backspace,
    Esc,
    PowerOff,
}

const KEYBOARD_BUFFER_SIZE: usize = 1024;
const DISPLAY_BUFFER_SIZE: usize = 1024;

// 40 x 24
pub struct Display {
    rx: Receiver<Tecla>,
}

impl Display {
    pub fn new(rx: Receiver<Tecla>) -> anyhow::Result<Self> {
        crossterm::execute!(std::io::stdout(), EnterAlternateScreen)?;
        crossterm::terminal::enable_raw_mode()?;

        Ok(Self { rx })
    }

    pub fn run(&mut self) {
        loop {
            if let Ok(x) = self.rx.recv() {
                match x {
                    Tecla::Printable(c) => {
                        write!(
                            std::io::stdout(),
                            "{}",
                            char::from_u32(c as u32).unwrap_or_default()
                        )
                        .ok();
                        std::io::stdout().flush().ok();
                    }
                    Tecla::Enter => {
                        write!(std::io::stdout(), "\r\n").ok();
                        std::io::stdout().flush().ok();
                    }
                    Tecla::Backspace => {}
                    Tecla::Esc => {}
                    Tecla::PowerOff => {
                        break;
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
    tx: Sender<Tecla>,
}

impl Keyboard {
    pub fn new(tx: Sender<Tecla>) -> Self {
        Self { tx }
    }

    pub fn run(&mut self) {
        let Self { tx } = self;
        loop {
            match crossterm::event::read().ok() {
                Some(Event::Key(KeyEvent {
                    code: KeyCode::Char(mut c),
                    ..
                })) => {
                    if c.is_ascii() {
                        c.make_ascii_uppercase();
                        tx.send(Tecla::Printable(c as u8)).ok();
                    }
                }
                Some(Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                })) => {
                    tx.send(Tecla::Enter).ok();
                }
                Some(Event::Key(KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                })) => {
                    tx.send(Tecla::Backspace).ok();
                }
                Some(Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                })) => {
                    tx.send(Tecla::Esc).ok();
                }
                Some(Event::Key(KeyEvent {
                    code: KeyCode::End, ..
                })) => {
                    tx.send(Tecla::PowerOff).ok();
                    break;
                }
                _ => {}
            }
        }
    }
}

pub fn keyboard_rx_tx() -> (Sender<Tecla>, Receiver<Tecla>) {
    crossbeam_channel::bounded::<Tecla>(KEYBOARD_BUFFER_SIZE)
}

pub fn display_rx_tx() -> (Sender<Tecla>, Receiver<Tecla>) {
    crossbeam_channel::bounded::<Tecla>(DISPLAY_BUFFER_SIZE)
}
