use std::fs::File;
use std::io::Read;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

const START_ADDRESS: u16 = 0x200;
const FONTSET_START_ADDRESS: u16 = 0x50;
const RAM: usize = 4096;

const FONTSET_SIZE: usize = 80;

const FONT_DATA: [u8; FONTSET_SIZE] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

struct Chip8 {
    v_reg: [u8; 16],
    ram: [u8; 4096],
    i_reg: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u16,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [u8; 16],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip_8 = Self {
            v_reg: [0; 16],
            ram: [0; RAM],
            i_reg: 0,
            pc: START_ADDRESS,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
        };

        // load fonts into memory
        chip_8.ram[..FONTSET_SIZE].copy_from_slice(&FONT_DATA);

        chip_8
    }

    pub fn load(&mut self, path: &str) {
        let mut file = File::open(path).expect("Cannot access file path.");

        let mut rom_buffer = [0u8; 3584];
        file.read(&mut rom_buffer).unwrap_or(0);

        // change ??? bad source
        for (i, &byte) in rom_buffer.iter().enumerate() {
            let addr = START_ADDRESS + i;
            if addr < 4096 {
                self.ram[START_ADDRESS + i] = byte;
            } else {
                break;
            }
        }
    }

    pub fn push(&mut self, adr: u16) {
        self.stack[self.sp as usize] = adr;
        self.sp += 1;
    }

    pub fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self) {
        let opcode = self.get_opcode();

        self.execute(opcode);
    }

    fn get_opcode(&mut self) -> u16 {
        let high_byte = self.ram[self.pc as usize] as u16;
        let low_byte = self.ram[(self.pc + 1) as usize] as u16;
        let opcode = (high_byte << 8) | low_byte;
        self.pc += 2;
        opcode
    }

    fn execute(&mut self, opcode: u16) {
        let b4_1 = (opcode & 0xF000) >> 12;
        let b4_2 = (opcode & 0x0F00) >> 8;
        let b4_3 = (opcode & 0x00F0) >> 4;
        let b4_4 = opcode & 0x000F;

        match (b4_1, b4_2, b4_3, b4_4) {
            (_, _, _, _) => {}
        }
    }
}

impl Chip8 {
    /// CLS: clear the display buffer
    fn op_00e0(&mut self) {
        for i in 0..self.screen.len() {
            self.screen[i] = 0;
        }
    }

    /// RET: return from subroutine
    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp];
    }

    /// JP addr: jump to nnn
    fn op_1nnn(&mut self) {}

    /// CALL addr: call subroutine at nnn
    fn op_2nnn(&mut self) {}

    /// SE Vx, byte: skip next instruction if Vx == kk
    fn op_3xkk(&mut self) {}

    /// SNE Vx, byte: skip next instruction if Vx != kk
    fn op_4xkk(&mut self) {}

    /// SE Vx, Vy: skip next instruction if Vx == Vy
    fn op_5xy0(&mut self) {}

    /// LD Vx, byte: set Vx = kk
    fn op_6xkk(&mut self) {}

    /// ADD Vx, byte: add kk to Vx
    fn op_7xkk(&mut self) {}

    /// LD Vx, Vy: set Vx = Vy
    fn op_8xy0(&mut self) {}

    /// OR Vx, Vy: set Vx = Vx OR Vy
    fn op_8xy1(&mut self) {}

    /// AND Vx, Vy: set Vx = Vx AND Vy
    fn op_8xy2(&mut self) {}

    /// XOR Vx, Vy: set Vx = Vx XOR Vy
    fn op_8xy3(&mut self) {}

    /// ADD Vx, Vy: set Vx = Vx + Vy and set VF = carry bit
    fn op_8xy4(&mut self) {}

    /// SUB Vx, Vy: set Vx = Vx - Vy and set VF = ~(borrow bit)
    fn op_8xy5(&mut self) {}

    /// SHR Vx: shift Vx one bit right, save least-significant bit in VF
    fn op_8xy6(&mut self) {}
}

fn main() {
    println!("Hello, world!");
}
