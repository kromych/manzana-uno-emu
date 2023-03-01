use crate::terminal::Tecla;

use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;

use yamos6502::MemoryError;
use yamos6502::MAX_MEMORY_SIZE;

// KBD b7..b0 are inputs, b6..b0 is ASCII input, b7 is constant high
// DSP b6..b0 are outputs, b7 is input

// PIA.A keyboard input
const KBD: u16 = 0xd010;
// PIA.A keyboard control register
const KBDCR: u16 = 0xd011;
// PIA.B display output register
const DSP: u16 = 0xd012;
// PIA.B display control register
const DSPCR: u16 = 0xd013;

const BS: u8 = 0xDF;
const CR: u8 = 0x8D;
const ESC: u8 = 0x9B;

const INIT_MEMORY_VALUE: u8 = 0xff;
const INIT_STACK_POINTER: u8 = 0xfd;
const ROM_START: u16 = 0xff00;

const WOZMON: &[u8] = include_bytes!("wozmon.bin");

pub struct Board {
    keyboard_in: Receiver<Tecla>,
    display_out: Sender<Tecla>,
    poweroff_out: Sender<()>,
    bytes: [u8; MAX_MEMORY_SIZE],
    rom_start: u16,
}

impl yamos6502::Memory for Board {
    fn write(&mut self, addr: u16, value: u8) -> Result<(), MemoryError> {
        if addr >= self.rom_start {
            return Err(MemoryError::ReadOnlyAddress(addr));
        }

        let mut value = value;
        match addr {
            DSP => {
                self.display_out.send(Tecla::Char(value & 0b0111_1111)).ok();
                // Clear the bit 7 to indicate that the operation completed
                value &= 0b0111_1111;
            }
            DSPCR | KBDCR => {
                // Is set to 0b1010_0111 normally
            }
            _ => {
                //
            }
        }
        self.bytes[addr as usize] = value;

        Ok(())
    }

    fn read(&self, addr: u16) -> Result<u8, MemoryError> {
        let data = match addr {
            KBD => {
                let t = self.keyboard_in.recv().unwrap();
                match t {
                    Tecla::Char(c) => c | 0b1000_0000,
                    Tecla::Enter => todo!(),
                    Tecla::Esc => todo!(),
                    Tecla::ClearScreen => todo!(),
                    Tecla::Reset => todo!(),
                    Tecla::PowerOff => todo!(),
                }
            }
            KBDCR => self.bytes[addr as usize] | ((self.keyboard_in.is_empty() as u8) << 7),
            _ => self.bytes[addr as usize],
        };

        Ok(data)
    }
}

impl Board {
    pub fn new(
        keyboard_in: Receiver<Tecla>,
        display_out: Sender<Tecla>,
        poweroff_out: Sender<()>,
    ) -> Self {
        let mut bytes = [INIT_MEMORY_VALUE; MAX_MEMORY_SIZE];
        bytes[ROM_START as usize..].copy_from_slice(WOZMON);

        Self {
            keyboard_in,
            display_out,
            poweroff_out,
            bytes,
            rom_start: ROM_START,
        }
    }
}

pub struct Manzana {
    poweroff_in: Receiver<()>,
    cpu: yamos6502::Mos6502<Board>,
}

impl Manzana {
    pub fn new(
        keyboard_in: Receiver<Tecla>,
        display_out: Sender<Tecla>,
        poweroff_in: Receiver<()>,
        poweroff_out: Sender<()>,
    ) -> Self {
        let board = Board::new(keyboard_in, display_out, poweroff_out);
        let cpu = yamos6502::Mos6502::new(board, yamos6502::StackWraparound::Disallow);

        Self { poweroff_in, cpu }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let Self { cpu, poweroff_in } = self;

        cpu.set_reset_pending();
        *cpu.registers_mut().reg_mut(yamos6502::Register::S) = INIT_STACK_POINTER;
        loop {
            cpu.run()?;
            if poweroff_in.try_recv().is_ok() {
                break;
            }
        }

        Ok(())
    }
}

pub fn poweroff() -> (Sender<()>, Receiver<()>) {
    crossbeam_channel::bounded::<()>(0)
}
