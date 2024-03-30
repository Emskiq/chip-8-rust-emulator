# Chip-8 Rust emulator

Yet another [Chip-8](https://en.m.wikipedia.org/wiki/CHIP-8) emulator written in Rust, marking the third step of my _Rust auditor_ journey. This project is following the previous [PngMe](https://github.com/Emskiq/pngme/tree/main) project, which actually suggested doing this emulator.

![](tetris-game.gif)

I would say this project is perfect for you, *if you have previous experience with programming*. [Laurence Muller](https://multigesture.net/about/) couldn't summurize it better in his [blogpost](https://multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/)<br>
- Don’t use this project as a way to learn how to program. (If bitwise operations confuse you, study them first)


# Overview
This project is ideal for anyone who is already familiar with Rust and programming in general. I would compare its complexity to that of creating a small website using a framework like Django.

It comes highly recommended by the creator of the PngMe project, which (in my opinion) is _nearly perfect_ intermediate Rust project. However, unlike PngMe, this project lacks straightforward instructions and unit tests.

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

_You can set the scale in the options by typing `-s 16` e.g._

# Basic logic flow

- ### The logic behind decoding

- ### How is memory handled (not that hard)

- ### Keyboard

- ### Graphics


# Resources and possibly some guidance 


