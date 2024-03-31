# Chip-8 Rust emulator

Yet another [Chip-8](https://en.m.wikipedia.org/wiki/CHIP-8) emulator written in Rust, marking the third step of my _Rust auditor_ journey. This project is following the previous [PngMe](https://github.com/Emskiq/pngme/tree/main) project, which actually suggested doing this emulator.

![](tetris-game.gif)

I would say this project is perfect for you, *if you have previous experience with programming*. [Laurence Muller](https://multigesture.net/about/) couldn't summurize it better in his [blogpost](https://multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/)<br>
- Don’t use this project as a way to learn how to program. (If bitwise operations confuse you, study them first)

# Overview
This project is ideal for anyone who is already familiar with Rust and programming in general. I would compare its complexity to that of creating a small website using a framework like Django.

It comes highly recommended by the creator of the PngMe project, which (in my opinion) is _nearly perfect_ intermediate Rust project. However, unlike PngMe, this project lacks straightforward instructions and unit tests.

You can still test your implementation though and **I highly recommend doing it**. [These testsuites](https://github.com/Timendus/chip8-test-suite) are part of an excellent repository that will guide you with finalizing your code and finding any missed bugs.

The project delves into various topics in Rust and programming, including:
- Understanding the overall operation of a processor with ROM and a set of opcodes
- Handling graphics and user inputs
- Managing graphics, audio, and frame updates
- Working with memory stacks

### Overall score: 8 - 8.5

# Setup
### Prerequisites
- Linux OS, as the project has been developed on Ubuntu.
- [SDL2](https://en.wikipedia.org/wiki/Simple_DirectMedia_Layer) installed
- Rust installed (obviously 😁)

### Instalation steps
0. Clone the repository and install SDL
   
    ```bash
    sudo apt-get install libsdl2-dev
    ```

1. Build it locally

    ```bash
    cargo build
    ```
2. Run it
    ```bash
   ./target/debug/chip-8 [OPTIONS] <PROGRAM_FILE>
    ```

_You can set the scale in the options by typing `-s 8` e.g. (by default is `16`)_

# Implementation

There are plenty of resources available that provide excellent pseudo code, which can guide you through completing the emulator. In this section, I'll provide a general overview of the main processes that occur behind the scenes while the emulator is functioning, along with sharing insights, resources, and problems I've encountered during development.

I highly recommend checking out these two excellent resources that essentially provide pseudo-code for implementing the emulator:
- [C/C++ emulator with pseudo code and logic](https://multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/)
- [Rust emulator with pseudo code and logic](https://dhole.github.io/post/chip8_emu_1/)

## My Main Takeaways

- ### Game Loop/Cycle

Initially, I implemented the basic game cycle/loop without considering factors such as framerate. While this approach may suffice for getting started, it's crucial to keep in mind that testing may reveal issues with the main processor cycle. Consider adding intentional delays/overtime to regulate the emulator's processing speed
(_Check the while loop in the main function_)
```rust
pub fn cycle(&mut self, key: u16) -> Result<()> {
  // store pressed key
  self.keypad = key;

  // update timers

  // --- Execution of an instruction in a FRAME
  self.time += FRAME_TIME;
  while self.time > 0 {
      // get/fetch instruction
      // decode operation code of instruction
      // execute instruction + get overtime that it takes to be executed originally
      //  --> This function can return overtime that will slow down the emulator
  }
  Ok(())
}
```

- ### Memory handling and Opcode decoding/exection

Personally, this part was fairly easy for me. However, it was also the one that led to bugs persisting until the end of development, mainly due to incorrect execution of the decoded opcodes.

Ensure that you properly load the chipset into memory and thoroughly check the implementation of opcodes to avoid such issues.

What can help you here are the [testsuites](https://github.com/Timendus/chip8-test-suite) mentioned above.

```rust
pub fn new(program: PathBuf) -> Result<Self, LoadInMemoryError>  {
  let mut emulator = Chip8{..Default::default()};

  emulator.load_font_set_in_memory();
  emulator.load_program_in_memory(program)?;

  Ok(emulator)
}
```

   - ### Keyboard and Graphics
   
Handling key input and displaying graphics were among the most challenging aspects of development. Thorough articles on how to tackle these tasks were scarce, and I couldn't find any third-party suggestions either.

Ultimately, I opted to use the implementation from the [Rust example repository](https://dhole.github.io/post/chip8_emu_1/) mentioned above. This implementation proved to be solid and the best resource available in terms of explanation and ease of adapting similar implementations for my emulator code.

The implementation utilizes SDL2, which offers both key detection functionality and graphics handling. It's somewhat _magical_ how SDL2 precisely handles graphics, but the implementation worked flawlessly with my code (which was heavily influenced by the Rust repository mentioned earlier 😅).


# Resources

Here is a list of helpful resources that I actively referred to during development:

- [CHIP-8 Wiki Documentation](https://en.m.wikipedia.org/wiki/CHIP-8) and [Alternative Documentation](https://github.com/mattmikolay/chip-8/wiki/CHIP‐8-Technical-Reference): The processor documentation/technical reference is always your main resource when writing an emulator.
- [Rust Example Repo](https://dhole.github.io/post/chip8_emu_1/) Without this repository I was _fucked_.
- [Chip-8 emulator testsuite Repo](https://github.com/Timendus/chip8-test-suite) Without these tests I _couldn't finish_ the project. 
- [SDL2 Crate](https://github.com/Rust-SDL2/rust-sdl2): This crate handles the keyboard and graphics of the emulator, as mentioned earlier.
- [CLAP Documentation on accepting arguments](https://docs.rs/clap/latest/clap/_derive/_tutorial/chapter_2/index.html): CLAP was perfect for polishing my project by defining the options and arguments that I wanted to pass to the executable and execute the program.
- [Operators and Symbols in Rust](https://doc.rust-lang.org/book/appendix-02-operators.html): Understanding symbols in Rust is crucial when working with bits and bytes.
- [Wrapped Add/Sub in Rust](https://doc.rust-lang.org/stable/std/?search=wrapping): Understanding why Rust promotes privacy and is used in blockchains like Solana.
- [Enum to String Crate](https://crates.io/crates/enum-stringify) and [Logger Crate](https://docs.rs/crate/env_logger/latest)
- [Termion](https://github.com/redox-os/termion/blob/master/examples/keys.rs): Although not used in the final implementation, it was fun experimenting with this amazing crate.
