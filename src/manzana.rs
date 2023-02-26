use crate::terminal::Tecla;

use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;

pub struct Manzana {
    kbd: Receiver<Tecla>,
    dsp: Sender<Tecla>,
}

impl Manzana {
    pub fn new(kbd: Receiver<Tecla>, dsp: Sender<Tecla>) -> Self {
        Self { kbd, dsp }
    }

    pub fn run(&mut self) {
        let Self { kbd, dsp } = self;
        loop {
            if let Ok(x) = kbd.recv() {
                dsp.send(x).ok();
                if x == Tecla::PowerOff {
                    break;
                }
            }
        }
    }
}
