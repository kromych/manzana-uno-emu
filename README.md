# Apple I emulator

## Usage

```text
Usage: manzana-uno-emu [OPTIONS] [MEM_FILE_LIST]

Arguments:
  [MEM_FILE_LIST]
          Paths to the files to seed the memory with.

          Format is (path[:load_addr_hex_no_0x],)+, load addresses must increase, and the loaded files must not overlap.

Options:
      --log-level <LOG_LEVEL>
          Logging level

          [default: info]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

Either of `Esc`, `Home`, `End` keys make the emulator exit.

## Examples

### Woz Monitor

The Woz monitor is always loaded at `0xff00`. It allows to inspect
memory contents as well as enter new programs and run them.

### Apple 1 BASIC

Run the emulator with `apple1basic.bin` loaded at `0xe000`:

```sh
cargo run --release -- roms/apple1basic.bin:e000
```

Inside the emulator, issue `E000R` to run the BASIC interpreter.
Here is a sample program you might enter

```basic
10 FOR I=1 TO 10
20 PRINT "HELLO #", I
30 NEXT I
40 END
```

and run it with

```basic
RUN
```

### Apple 30 years

Run the emulator with `apple30.bin` loaded at `0x0280`:

```sh
cargo run --release -- roms/apple30.bin:280
```

Inside the emulator, issue `280R` to run the demo.

## Other resources you might find interesting

* [Applefritter](https://www.applefritter.com)
* [Woz Monitor manual](https://www.sbprojects.net/projects/apple1/wozmon.php)
* [Apple 1 BASIC manual](https://archive.org/stream/apple1_basic_manual/apple1_basic_manual_djvu.txt)
