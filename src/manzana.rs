use crate::terminal::Tecla;

use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;

pub struct Manzana {
    keyboard_in: Receiver<Tecla>,
    display_out: Sender<Tecla>,
}

impl Manzana {
    pub fn new(keyboard_in: Receiver<Tecla>, display_out: Sender<Tecla>) -> Self {
        Self {
            keyboard_in,
            display_out,
        }
    }

    pub fn run(&mut self) {
        let Self {
            keyboard_in,
            display_out,
        } = self;
        loop {
            if let Ok(x) = keyboard_in.recv() {
                display_out.send(x).ok();
                if x == Tecla::PowerOff {
                    break;
                }
            }
        }
    }
}
