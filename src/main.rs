#![allow(clippy::struct_excessive_bools)]

mod error;
mod quirk;
mod draw;
mod cpu;

use crate::{
    error::EmuError,
    quirk::Quirks,
    cpu::Cpu,
};
use std::{
    fs::File, 
    io::Read,
    time::{Duration, Instant},
};
use draw::Renderer;
use sdl2::{
    event::Event, 
    EventPump,
};
use frand::Rand;
use clap::Parser;

const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,
    0x20, 0x60, 0x20, 0x20, 0x70,
    0xF0, 0x10, 0xF0, 0x80, 0xF0,
    0xF0, 0x10, 0xF0, 0x10, 0xF0,
    0x90, 0x90, 0xF0, 0x10, 0x10,
    0xF0, 0x80, 0xF0, 0x10, 0xF0,
    0xF0, 0x80, 0xF0, 0x90, 0xF0,
    0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0,
    0xF0, 0x90, 0xF0, 0x10, 0xF0,
    0xF0, 0x90, 0xF0, 0x90, 0x90,
    0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0,
    0xE0, 0x90, 0x90, 0x90, 0xE0,
    0xF0, 0x80, 0xF0, 0x80, 0xF0,
    0xF0, 0x80, 0xF0, 0x80, 0x80
];

/// CHIP-8 Interpreter
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The path to the ROM
    rom: String,
    /// The instructions per second
    #[arg(short, long, default_value_t = 700)]
    speed: u32,
}

fn init() -> Result<(Renderer, Rand, EventPump), EmuError> {
    let sdl_context = sdl2::init().map_err(EmuError::Sdl)?;
    let video_subsystem = sdl_context.video().map_err(EmuError::Sdl)?;

    let window = video_subsystem
        .window("CHIP-8 Emulator", 1080, 512)
        .position_centered()
        .build()?;

    let renderer = Renderer::new(window)?;
    let rng = Rand::new();
    let event_pump = sdl_context.event_pump().map_err(EmuError::Sdl)?;
    Ok((renderer, rng, event_pump))
}

fn main() -> Result<(), EmuError> {
    let args = Args::parse();
    let (mut renderer, mut rng, mut event_pump) = init()?;
    let mut cpu = Cpu::new()?;
    let quirks = Quirks::new();

    File::open(args.rom)?.read_to_end(&mut cpu.rom)?;

    for (i, byte) in cpu.rom.iter().enumerate() {
        cpu.memory[i + 0x200] = *byte;
    }
    for (i, char) in FONT.iter().enumerate() {
        cpu.memory[i] = *char;
    }

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    return Ok(())
                }
                Event::KeyDown { keycode: Some(key), .. } => {
                    if let Some(key) = Cpu::match_key(key) {
                        cpu.keys[key] = true;
                    }
                },
                Event::KeyUp { keycode: Some(key), .. } => {
                    if let Some(key) = Cpu::match_key(key) {
                        cpu.keys[key] = false;
                    }
                },
                _ => (),
            }
        }
        cpu.tick_timers(Instant::now());
        fetch(&mut cpu);
        if let Err(e) = decode(&mut cpu, &quirks, &mut rng, &mut renderer) {
            eprintln!("Got error: {e}");
        }
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / args.speed));
    }
}

fn fetch(cpu: &mut Cpu) {
    cpu.opcode = u16::from_be_bytes([cpu.memory[cpu.pc as usize], cpu.memory[cpu.pc as usize + 1]]);
    cpu.pc += 2;
}

fn decode(cpu: &mut Cpu, quirks: &Quirks, rng: &mut Rand, renderer: &mut Renderer) -> Result<(), EmuError> {
    let x = ((cpu.opcode & 0x0F00) >> 8) as usize;
    let y = ((cpu.opcode & 0x00F0) >> 4) as usize;
    match (cpu.opcode & 0xF000) >> 12 {
        0x0 => {
            match cpu.opcode & 0x00FF {
                0xE0 => {
                    cpu.display_buffer.fill(false);
                    renderer.draw(cpu)?;
                },
                0xEE => cpu.pc = cpu.stack.pop().ok_or(EmuError::Stack("Tried to pop from stack but stack is empty".to_owned()))?,
                _ => return Err(EmuError::Invalid(cpu.opcode)),
            }
        },
        0x1 => cpu.pc = cpu.opcode & 0x0FFF,
        0x2 => {
            if cpu.stack.len() <= 16 {
                cpu.stack.push(cpu.pc);
                cpu.pc = cpu.opcode & 0x0FFF;
            } else {
                return Err(EmuError::Stack("Tried to push to stack but stack is at maximum length".to_owned()));
            }
        },
        0x3 => cpu.skip_instruction(u16::from(cpu.v[x]) == cpu.opcode & 0x00FF),
        0x4 => cpu.skip_instruction(u16::from(cpu.v[x]) != cpu.opcode & 0x00FF),
        0x5 => cpu.skip_instruction(cpu.v[x] == cpu.v[y]),
        0x6 => (cpu.v[x], _) = 0u8.overflowing_add((cpu.opcode & 0x00FF) as u8),
        0x7 => (cpu.v[x], _) = cpu.v[x].overflowing_add((cpu.opcode & 0x00FF) as u8),
        0x8 => decode_8(cpu, quirks, x, y)?,
        0x9 => cpu.skip_instruction(cpu.v[x] != cpu.v[y]),
        0xA => cpu.i = cpu.opcode & 0x0FFF,
        0xB => {
            let i = quirks.jump(cpu, x);
            cpu.pc = (cpu.opcode & 0x0FFF) + u16::from(i);
        },
        0xC => {
            let random = rng.r#gen::<u8>();
            cpu.v[x] = random & (cpu.opcode & 0x00FF) as u8;
        },
        0xD => {
            let x = u16::from(cpu.v[x] % 64);
            let y = u16::from(cpu.v[y] % 32);
            cpu.v[0xF] = 0;
            for row in 0..(cpu.opcode & 0x000F) {
                for col in 0..8 {
                    if quirks.wrap || ((y + row < 32) && (x + col < 64)) {
                        let sprite_pixel = cpu.memory[(cpu.i + row) as usize] & (0x80 >> col);
                        let screen_pixel = &mut cpu.display_buffer[(((y + row) * 64) + x + col) as usize];
                        if sprite_pixel != 0 {
                            if *screen_pixel {
                                cpu.v[0xF] = 1;
                            }
                            *screen_pixel ^= true;
                        }
                    }
                }
            }
            renderer.draw(cpu)?;
        },
        0xE => {
            match cpu.opcode & 0x00FF {
                0x9E => cpu.skip_instruction(cpu.keys[cpu.v[x] as usize]),
                0xA1 => cpu.skip_instruction(!cpu.keys[cpu.v[x] as usize]),
                _ => return Err(EmuError::Invalid(cpu.opcode)),
            }
        },
        0xF => decode_f(cpu, quirks, x)?,
        _ => return Err(EmuError::Invalid(cpu.opcode)),
    }
    Ok(())
}

fn decode_8(cpu: &mut Cpu, quirks: &Quirks, x: usize, y: usize) -> Result<(), EmuError> {
    match cpu.opcode & 0x000F {
        0x0 => cpu.v[x] = cpu.v[y],
        0x1 => {
            cpu.v[x] |= cpu.v[y];
            if quirks.logic {
                cpu.v[0xF] = 0;
            }
        },
        0x2 => {
            cpu.v[x] &= cpu.v[y];
            if quirks.logic {
                cpu.v[0xF] = 0;
            }
        },
        0x3 => {
            cpu.v[x] ^= cpu.v[y];
            quirks.logic(cpu);
        },
        0x4 => {
            let (result, carry) = cpu.v[x].overflowing_add(cpu.v[y]);
            cpu.v[x] = result;
            cpu.set_flag_register(carry);
        },
        0x5 => {
            let (result, carry) = cpu.v[x].overflowing_sub(cpu.v[y]);
            cpu.v[x] = result;
            cpu.set_flag_register(!carry);
        },
        0x6 => {
            quirks.shift(cpu, x, y);
            let shifted_bit = cpu.v[x] & 1;
            cpu.v[x] >>= 1;
            cpu.v[0xF] = shifted_bit;
        },
        0x7 => {
            let (result, carry) = cpu.v[y].overflowing_sub(cpu.v[x]);
            cpu.v[x] = result;
            cpu.set_flag_register(!carry);
        },
        0xE => {
            quirks.shift(cpu, x, y);
            let shifted_bit = cpu.v[x] & 0x80;
            (cpu.v[x], _) = cpu.v[x].overflowing_shl(1);
            cpu.v[0xF] = u8::from(shifted_bit != 0);
        },
        _ => return Err(EmuError::Invalid(cpu.opcode)),
    }
    Ok(())
}

fn decode_f(cpu: &mut Cpu, quirks: &Quirks, x: usize) -> Result<(), EmuError> {
    match cpu.opcode & 0x00FF {
        0x07 => cpu.v[x] = cpu.delay_timer,
        0x15 => cpu.delay_timer = cpu.v[x],
        0x18 => cpu.sound_timer = cpu.v[x],
        0x1E => cpu.i += u16::from(cpu.v[x]),
        0x0A => {
            if let Some(key) = cpu.keys.iter().position(|&x| x) {
                if !cpu.key_state {
                    cpu.v[x] = u8::try_from(key)?;
                    cpu.key_state = true;
                }
                cpu.pc -= 2;
            } else if cpu.key_state {
                cpu.key_state = false;
            } else {
                cpu.pc -= 2;
            }
        },
        0x29 => cpu.i = u16::from(cpu.v[x]) * 5,
        0x33 => {
            cpu.memory[cpu.i as usize] = cpu.v[x] / 100;
            cpu.memory[cpu.i as usize + 1] = (cpu.v[x] / 10) % 10;
            cpu.memory[cpu.i as usize + 2] = cpu.v[x] % 10;
        },
        0x55 => {
            for i in 0..=x { // x+1 cause its vX inclusive
                cpu.memory[cpu.i as usize + i] = cpu.v[i];
            }
            quirks.memory_increment_by_x(cpu, x)?;
            quirks.memory_leave_i_unchanged(cpu, x)?;
        },
        0x65 => {
            for i in 0..=x {
                cpu.v[i] = cpu.memory[cpu.i as usize + i];
            }
            quirks.memory_increment_by_x(cpu, x)?;
            quirks.memory_leave_i_unchanged(cpu, x)?;
        }
        _ => return Err(EmuError::Invalid(cpu.opcode)),
    }
    Ok(())
}