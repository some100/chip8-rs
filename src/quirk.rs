use crate::{cpu::Cpu, EmuError};

pub struct Quirks {
    pub shift: bool,
    pub memory_increment_by_x: bool,
    pub memory_leave_i_unchanged: bool,
    pub wrap: bool,
    pub jump: bool,
    pub logic: bool,
}

impl Quirks {
    pub fn new() -> Quirks {
        Quirks {
            shift: true,
            memory_increment_by_x: false,
            memory_leave_i_unchanged: true,
            wrap: false,
            jump: true,
            logic: false,
        }
    }
    pub fn shift(&self, cpu: &mut Cpu, x: usize, y: usize) {
        if !self.shift {
            cpu.v[x] = cpu.v[y];
        }
    }
    pub fn memory_increment_by_x(&self, cpu: &mut Cpu, x: usize) -> Result<(), EmuError> {
        if self.memory_increment_by_x && !self.memory_leave_i_unchanged {
            cpu.i += u16::try_from(x)?;
        }
        Ok(())
    }
    pub fn memory_leave_i_unchanged(&self, cpu: &mut Cpu, x: usize) -> Result<(), EmuError> {
        if !self.memory_increment_by_x && !self.memory_leave_i_unchanged {
            cpu.i += u16::try_from(x)? + 1;
        }
        Ok(())
    }
    pub fn jump(&self, cpu: &mut Cpu, x: usize) -> u8 {
        if self.jump {
            cpu.v[x]
        } else {
            cpu.v[0]
        }
    }
    pub fn logic(&self, cpu: &mut Cpu) {
        if self.logic {
            cpu.v[0xF] = 0;
        }
    }
}