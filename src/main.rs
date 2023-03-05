use clap::Parser;

mod manzana;
mod terminal;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Paths to the files to seed the memory with.
    ///
    /// Format is (path[:load_addr_hex_no_0x],)+, load addresses must increase,
    /// and the loaded files must not overlap.
    mem_file_list: Option<String>,
    /// Logging level
    #[clap(long, default_value = "info")]
    log_level: tracing::Level,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let trace_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .read(false)
        .truncate(false)
        .open("manzana-trace.log")?;
    tracing_subscriber::fmt()
        .with_max_level(args.log_level)
        .with_target(false)
        .with_writer(trace_file)
        .init();

    let (keyboard_out, keyboard_in) = terminal::keyboard_ports();
    let (display_out, display_in) = terminal::display_ports();
    let mut keyboard = terminal::Keyboard::new(keyboard_out);
    let mut display = terminal::Display::new(display_in)?;

    let bytes = seed_memory(&args)?;
    let mut manzana = manzana::Manzana::new(keyboard_in, display_out, bytes);

    let keyboard = std::thread::spawn(move || keyboard.run());
    let display = std::thread::spawn(move || display.run());

    manzana.run()?;

    keyboard.join().unwrap();
    display.join().unwrap();

    Ok(())
}

fn seed_memory(args: &Args) -> anyhow::Result<Vec<u8>> {
    const INIT_MEMORY_VALUE: u8 = 0x00;

    let mut memory = vec![];
    if let Some(file_list) = &args.mem_file_list {
        for file_path_addr in file_list.split(',') {
            let mut file_path_addr = file_path_addr.split(':');

            let file_path = file_path_addr.next();
            if file_path.is_none() {
                anyhow::bail!("Unexpected format of the memory file list");
            }
            let file_path = file_path.unwrap();
            tracing::info!("Reading memory contents from {file_path}");
            let chunk = std::fs::read(file_path)?;
            tracing::info!("Read 0x{:04x} bytes", chunk.len());

            if let Some(addr) = file_path_addr.next() {
                if let Ok(addr) = u16::from_str_radix(addr, 16) {
                    if memory.len() > addr as usize {
                        anyhow::bail!("Load addresses must increase");
                    }
                    // Fill the gap
                    memory
                        .extend_from_slice(&vec![INIT_MEMORY_VALUE; addr as usize - memory.len()]);
                } else {
                    anyhow::bail!(
                        "Load address {} isn't an unadorned 16-bit hex number (0000-ffff)",
                        addr
                    );
                }
            }
            tracing::info!("Loading at 0x{:04x}", memory.len());
            memory.extend_from_slice(&chunk);
        }

        if memory.len() > yamos6502::MAX_MEMORY_SIZE {
            anyhow::bail!(
                "Loaded 0x{:04x} bytes, maximum memory size is 0x{:04x} bytes",
                memory.len(),
                yamos6502::MAX_MEMORY_SIZE
            );
        }
    }

    // Fill the gap
    memory.extend_from_slice(&vec![
        INIT_MEMORY_VALUE;
        yamos6502::MAX_MEMORY_SIZE - memory.len()
    ]);

    Ok(memory)
}
