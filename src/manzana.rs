use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;

use crate::terminal::Tecla;
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
// PIA.B display output register alternative address
// that works due to incomplete decoding
const DSP_ALT: u16 = 0xd0f2;
// PIA.B display control register
const DSPCR: u16 = 0xd013;

const INIT_STACK_POINTER: u8 = 0xfd;
const ROM_START: u16 = 0xff00;

const WOZMON: &[u8] = include_bytes!("../roms/wozmon.bin");

pub struct Board {
    keyboard_in: Receiver<Tecla>,
    display_out: Sender<Tecla>,
    power_off_out: Sender<()>,
    bytes: Vec<u8>,
    rom_start: u16,
}

impl yamos6502::Memory for Board {
    fn write(&mut self, addr: u16, value: u8) -> Result<(), MemoryError> {
        if addr >= self.rom_start {
            return Err(MemoryError::ReadOnlyAddress(addr));
        }

        let mut value = value;
        match addr {
            DSP | DSP_ALT => {
                self.display_out.send(Tecla::Char(value)).ok();
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

        tracing::trace!("Wrote {value:02x} at {addr:04x}");

        Ok(())
    }

    fn read(&mut self, addr: u16) -> Result<u8, MemoryError> {
        let mut update_kbd_cr = || {
            if self.keyboard_in.is_empty() {
                self.bytes[KBDCR as usize] &= 0b0111_1111;
            } else {
                self.bytes[KBDCR as usize] |= 0b1000_0000;
            }
        };

        match addr {
            KBD => {
                if let Ok(tecla) = self.keyboard_in.recv() {
                    update_kbd_cr();

                    match tecla {
                        Tecla::Char(data) => {
                            self.bytes[addr as usize] = data;
                        }
                        Tecla::PowerOff => {
                            self.power_off_out.send(()).ok();
                            self.display_out.send(tecla).ok();
                        }
                    }
                }
            }
            KBDCR => {
                update_kbd_cr();
            }
            _ => {}
        };

        let data = self.bytes[addr as usize];

        tracing::trace!("Read {data:02x} at {addr:04x}");

        Ok(data)
    }
}

impl Board {
    pub fn new(
        keyboard_in: Receiver<Tecla>,
        display_out: Sender<Tecla>,
        power_off_out: Sender<()>,
        mut bytes: Vec<u8>,
    ) -> Self {
        assert!(bytes.len() == MAX_MEMORY_SIZE);

        {
            let bytes = bytes.as_mut_slice();
            bytes[ROM_START as usize..].copy_from_slice(WOZMON);

            bytes[KBD as usize] = 0;
            bytes[KBDCR as usize] = 0;
            bytes[DSP as usize] = 0;
            bytes[DSP_ALT as usize] = 0;
            bytes[DSPCR as usize] = 0;
        }

        Self {
            keyboard_in,
            display_out,
            bytes,
            rom_start: ROM_START,
            power_off_out,
        }
    }
}

pub struct Manzana {
    cpu: yamos6502::Mos6502<Board>,
    power_off_in: Receiver<()>,
}

impl Manzana {
    pub fn new(keyboard_in: Receiver<Tecla>, display_out: Sender<Tecla>, bytes: Vec<u8>) -> Self {
        let (power_off_out, power_off_in) = crossbeam_channel::bounded(1);
        let board = Board::new(keyboard_in, display_out, power_off_out, bytes);
        let cpu = yamos6502::Mos6502::new(board, yamos6502::StackWraparound::Allow);

        Self { cpu, power_off_in }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let Self { cpu, power_off_in } = self;

        cpu.set_reset_pending();
        *cpu.registers_mut().reg_mut(yamos6502::Register::S) = INIT_STACK_POINTER;
        let mut instr_emulated = 0;

        tracing::info!("Running Apple I emulator");
        loop {
            let run_exit = cpu.run()?;
            tracing::debug!("Run exit {:x?}, registers {:x?}", run_exit, cpu.registers());

            if power_off_in.is_full() {
                break;
            }

            if run_exit == yamos6502::RunExit::Executed(yamos6502::Insn::BRK) {
                // Reset on BRK
                cpu.set_reset_pending();
            }

            // Very arbitrary
            instr_emulated += 1;
            if instr_emulated & 0x1f == 0 {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }

        tracing::info!("Instruction emulated: {instr_emulated}");

        Ok(())
    }
}
