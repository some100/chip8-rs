use crate::EmuError;
use std::time::Instant;
use rodio::{source::SineWave, OutputStream, OutputStreamHandle, Sink};
use sdl2::keyboard::Keycode;

pub struct Cpu {
    pub rom: Vec<u8>,
    pub memory: [u8; 0x1000],
    pub display_buffer: [bool; 0x800],
    pub pc: u16,
    pub i: u16,
    pub stack: Vec<u16>,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub v: [u8; 0x10],
    pub keys: [bool; 0x10],
    pub key_state: bool,
    pub opcode: u16,
    past_time: Instant,
    _stream_handle: OutputStreamHandle,
    sink: Sink,
}

impl Cpu {
    pub fn new() -> Result<Cpu, EmuError> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        Ok(Cpu {
            rom: Vec::new(),
            memory: [0; 0x1000],
            display_buffer: [false; 0x800],
            pc: 0x200,
            i: 0,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            v: [0; 0x10],
            keys: [false; 0x10],
            key_state: false,
            opcode: 0x0000,
            past_time: Instant::now(),
            _stream_handle: stream_handle,
            sink,
        })
    }
    pub fn tick_timers(&mut self, now_time: Instant) {
        if now_time.duration_since(self.past_time).as_micros() > 16670 {
            self.delay_timer = self.delay_timer.saturating_sub(1);
            self.sound_timer = self.sound_timer.saturating_sub(1);
            if self.sound_timer != 0 {
                let beep = SineWave::new(440.0);
                self.sink.append(beep);
            } else {
                self.sink.stop();
            }
            self.past_time = Instant::now();
        }
    }
    pub fn set_flag_register(&mut self, condition: bool) {
        if condition {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }
    }
    pub fn skip_instruction(&mut self, condition: bool) {
        if condition {
            self.pc += 2;
        }
    }    
    pub fn match_key(key: Keycode) -> Option<usize> {
        match key {
            Keycode::NUM_1 => Some(0x1),
            Keycode::NUM_2 => Some(0x2),
            Keycode::NUM_3 => Some(0x3),
            Keycode::NUM_4 => Some(0xC),
            Keycode::Q => Some(0x4),
            Keycode::W => Some(0x5),
            Keycode::E => Some(0x6),
            Keycode::R => Some(0xD),
            Keycode::A => Some(0x7),
            Keycode::S => Some(0x8),
            Keycode::D => Some(0x9),
            Keycode::F => Some(0xE),
            Keycode::Z => Some(0xA),
            Keycode::X => Some(0x0),
            Keycode::C => Some(0xB),
            Keycode::V => Some(0xF),
            _ => None,
        }
    }
}
