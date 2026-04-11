use rand::Rng;
use std::borrow::Borrow;
use std::fs::File;
use std::io::Read;

pub const RAM_SIZE: usize = 4096;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Cpu {
    pub ram: [u8; RAM_SIZE],
    pub v: [u8; 16],
    pub i: u16,
    pub pc: u16,
    pub stack: [u16; 16],
    pub sp: u16,
    pub display: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],
    pub keypad: [bool; 16],
    pub delay_timer: u8,
    pub sound_timer: u8,
}

impl Cpu {
    pub fn new() -> Self {
        let mut cpu = Self {
            ram: [0; RAM_SIZE],
            v: [0; 16],
            i: 0,
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
            display: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            keypad: [false; 16],
            delay_timer: 0,
            sound_timer: 0,
        };

        cpu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        cpu
    }

    pub fn load_rom(&mut self, path: &str) {
        let mut file = File::open(path).expect("ROM file not found");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("Failed to read ROM");

        // Load the buffer into RAM starting at 0x200
        for (i, &byte) in buffer.iter().enumerate() {
            self.ram[0x200 + i] = byte;
        }
    }

    pub fn fetch(&mut self) -> u16 {
        let hi = self.ram[self.pc as usize] as u16;
        let lo = self.ram[(self.pc + 1) as usize] as u16;

        let opcode = (hi << 8) | lo;

        self.pc += 2;

        opcode
    }

    pub fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn tick(&mut self) {
        let opcode = self.fetch();

        let nibbles = (
            (opcode & 0xF000) >> 12,
            (opcode & 0x0F00) >> 8,
            (opcode & 0x00F0) >> 4,
            (opcode & 0x000F)
        );

        let nnn = opcode & 0x0FFF;
        let nn = (opcode & 0x00FF) as u8;

        match nibbles {
            (0, 0, 0xE, 0) => self.op_00e0(),
            (0, 0, 0xE, 0xE) => self.op_00ee(),
            (1, _, _, _) => self.op_1nnn(nnn),
            (2, _, _, _) => self.op_2nnn(nnn),
            (3, x, _, _) => self.op_3xnn(x as usize, nn),
            (4, x, _, _) => self.op_4xnn(x as usize, nn),
            (6, x, _, _) => self.op_6xnn(x as usize, nn),
            (7, x, _, _) => self.op_7xnn(x as usize, nn),
            (8, x, y, 0) => self.op_8xy0(x as usize, y as usize),
            (8, x, y, 1) => self.v[x as usize] |= self.v[y as usize],
            (8, x, y, 2) => self.v[x as usize] &= self.v[y as usize],
            (8, x, y, 3) => self.v[x as usize] ^= self.v[y as usize],
            (8, x, y, 4) => self.op_8xy4(x as usize, y as usize), // ADD
            (8, x, y, 5) => self.op_8xy5(x as usize, y as usize), // SUB
            (8, x, y, 6) => self.op_8xy6(x as usize, y as usize), // SHR
            (8, x, y, 7) => self.op_8xy7(x as usize, y as usize), // SUBN
            (8, x, y, 0xE) => self.op_8xye(x as usize, y as usize), // SHL
            (0xA, _, _, _) => self.op_annn(nnn),
            (0xC, x, _, _) => self.op_cxnn(x as usize, nn),
            (0xD, x, y, n) => self.op_dxyn(x as usize, y as usize, n as usize),
            (0xE, x, 9, 0xE) => {
                if self.keypad[self.v[x as usize] as usize] {
                    self.pc += 2
                }
            },
            (0xE, x, 0xA, 1) => {
                if !self.keypad[self.v[x as usize] as usize] {
                    self.pc += 2
                }
            },
            (0xF, x, 0, 7) => self.v[x as usize] = self.delay_timer,
            (0xF, x, 1, 5) => self.delay_timer = self.v[x as usize],
            (0xF, x, 1, 8) => self.sound_timer = self.v[x as usize],
            (0xF, x, 2, 9) => self.op_fx29(x as usize),
            (0xF, x, 3, 3) => self.op_fx33(x as usize),
            (0xF, x, 5, 5) => self.op_fx55(x as usize),
            (0xF, x, 6, 5) => self.op_fx65(x as usize),
            _ => println!("Unknown opcode: {:X}", opcode),
        }
    }

    fn op_00e0(&mut self) {
        self.display = [0; 64 * 32];
    }

    fn op_00ee(&mut self) {
        self.sp -= 1;

        self.pc = self.stack[self.sp as usize];
    }

    fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    fn op_2nnn(&mut self, nnn: u16) {
        self.stack[self.sp as usize] = self.pc;

        self.sp += 1;

        self.pc = nnn;
    }

    fn op_3xnn(&mut self, x: usize, nn: u8) {
        if self.v[x] == nn {
            self.pc += 2
        }
    }

    fn op_4xnn(&mut self, x: usize, nn: u8) {
        if self.v[x] != nn {
            self.pc += 2
        }
    }

    fn op_6xnn(&mut self, x: usize, nn: u8) {
        self.v[x] = nn;
    }

    fn op_7xnn(&mut self, x: usize, nn: u8) {
        self.v[x] = self.v[x].wrapping_add(nn);
    }

    fn op_8xy0(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
    }

    fn op_8xy4(&mut self, x: usize, y: usize) {
        let (val, overflow) = self.v[x].overflowing_add(self.v[y]);
        self.v[x] = val;

        self.v[0xF] = if overflow { 1 } else { 0 };
    }

    fn op_8xy5(&mut self, x: usize, y: usize) {
        let (val, borrow) = self.v[x].overflowing_sub(self.v[y]);
        self.v[x] = val;

        self.v[0xF] = if !borrow { 1 } else { 0 }
    }

    fn op_8xy6(&mut self, x: usize, _y: usize) {
        self.v[0xF] = self.v[x] & 0x1; // Store LSB in VF
        self.v[x] >>= 1;
    }
    fn op_8xy7(&mut self, x: usize, y: usize) {
        let (val, borrow) = self.v[y].overflowing_sub(self.v[x]);
        self.v[x] = val;
        self.v[0xF] = if !borrow { 1 } else { 0 };
    }

    fn op_8xye(&mut self, x: usize, _y: usize) {
        self.v[0xF] = (self.v[x] & 0x80) >> 7; // Store MSB in VF
        self.v[x] <<= 1;
    }
    
    fn op_annn(&mut self, nnn: u16) {
        self.i = nnn;
    }

    fn op_cxnn(&mut self, x: usize, nn: u8) {
        let mut rng = rand::thread_rng();
        let random_byte: u8 = rng.r#gen();

        self.v[x] = random_byte & nn;
    }

    fn op_dxyn(&mut self, x: usize, y: usize, height: usize) {
        let x_pos = (self.v[x] as usize) % 64;
        let y_pos = (self.v[y] as usize) % 32;

        self.v[0xF] = 0;

        for row in 0..height {
            let sprite_byte = self.ram[(self.i + row as u16) as usize];

            for col in 0..8 {
                if (sprite_byte & (0x80 >> col)) != 0 {
                    let sx = (x_pos + col) % 64;
                    let sy = (y_pos + row) % 32;
                    let index = sy * 64 + sx;

                    if self.display[index] == 1 {
                        self.v[0xF] = 1;
                        self.display[index] = 0;
                    } else {
                        self.display[index] = 1
                    }
                }
            }
        }
    }

    fn op_fx29(&mut self, x: usize) {
        let digit = self.v[x] as u16;

        self.i = digit * 5;
    }

    fn op_fx33(&mut self, x: usize) {
        let value = self.v[x];

        self.ram[self.i as usize] = value / 100;
        self.ram[(self.i + 1) as usize] = (value / 10) % 10;
        self.ram[(self.i + 2) as usize] = value % 10;
    }

    fn op_fx55(&mut self, x: usize) {
        for i in 0..=x {
            self.ram[(self.i + i as u16) as usize] = self.v[i];
        }
    }

    fn op_fx65(&mut self, x: usize) {
        for i in 0..=x {
            self.v[i] = self.ram[(self.i + i as u16) as usize];
        }
    }

}