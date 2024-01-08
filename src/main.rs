use rand::Rng;
use std::fs::File;
use std::io::Read;
use std::time::Duration;
use std::thread;

mod display;
mod input;

use display::Display;
use input::Input;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

const START_ADDRESS: u16 = 0x200;
const FONTSET_START_ADDRESS: u16 = 0x50;
const FONTSET_SPRITE_SIZE: u16 = 5;
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

#[derive(Debug)]
struct Chip8 {
    v_reg: [u8; 16],
    ram: [u8; 4096],
    i_reg: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u16,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [bool; 16],
    screen: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
    display_stale: bool,
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
            keypad: [false; 16],
            screen: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
            display_stale: false,
        };

        // load fonts into memory
        chip_8.ram[0..(FONTSET_START_ADDRESS as usize)].copy_from_slice(&FONT_DATA);

        chip_8
    }

    pub fn load(&mut self, path: &str) {
        let mut file = File::open(path).expect("Cannot access file path.");

        let mut rom_buffer = [0u8; 3584];
        file.read(&mut rom_buffer).unwrap_or(0);

        // there's a better way to do this...
        for (i, &byte) in rom_buffer.iter().enumerate() {
            let addr = START_ADDRESS as usize + i;
            if addr < 4096 {
                self.ram[addr] = byte;
            } else {
                break;
            }
        }
    }

    // pub fn push(&mut self, addr: u16) {
    //     self.stack[self.sp as usize] = addr;
    //     self.sp += 1;
    // }

    // pub fn pop(&mut self) -> u16 {
    //     self.sp -= 1;
    //     self.stack[self.sp as usize]
    // }

    pub fn tick(&mut self) {
        let opcode = self.get_opcode();
        self.pc += 2;
        self.execute(opcode);
        if self.delay_timer > 0 {
            self.delay_timer -= 1
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1
        }
    }

    fn get_opcode(&mut self) -> u16 {
        let high_byte = self.ram[self.pc as usize] as u16;
        let low_byte = self.ram[(self.pc + 1) as usize] as u16;
        let opcode = (high_byte << 8) | low_byte;
        opcode
    }

    fn execute(&mut self, opcode: u16) {
        let b0 = (opcode & 0xF000) >> 12;
        let b1 = (opcode & 0x0F00) >> 8;
        let b2 = (opcode & 0x00F0) >> 4;
        let b3 = opcode & 0x000F;

        let x = b1 as usize;
        let y = b2 as usize;
        let n = b3 as usize;
        let nnn = (opcode & 0x0FFF) as usize;
        let kk = (opcode & 0x00FF) as u8;

        match (b0, b1, b2, b3) {
            (0x00, 0, 0xE, 0) => self.op_00e0(),
            (0x00, 0, 0xE, 0xE) => self.op_00ee(),
            (0x01, _, _, _) => self.op_1nnn(nnn),
            (0x02, _, _, _) => self.op_2nnn(nnn),
            (0x03, _, _, _) => self.op_3xkk(x, kk),
            (0x04, _, _, _) => self.op_4xkk(x, kk),
            (0x05, _, _, 0x00) => self.op_5xy0(x, y),
            (0x06, _, _, _) => self.op_6xkk(x, kk),
            (0x07, _, _, _) => self.op_7xkk(x, kk),
            (0x08, _, _, 0x00) => self.op_8xy0(x, y),
            (0x08, _, _, 0x01) => self.op_8xy1(x, y),
            (0x08, _, _, 0x02) => self.op_8xy2(x, y),
            (0x08, _, _, 0x03) => self.op_8xy3(x, y),
            (0x08, _, _, 0x04) => self.op_8xy4(x, y),
            (0x08, _, _, 0x05) => self.op_8xy5(x, y),
            (0x08, _, _, 0x06) => self.op_8xy6(x, y),
            (0x08, _, _, 0x0E) => self.op_8xye(x, y),
            (0x08, _, _, 0x07) => self.op_8xy7(x, y),
            (0x09, _, _, _) => self.op_9xy0(x, y),
            (0x0A, _, _, _) => self.op_annn(nnn),
            (0x0B, _, _, _) => self.op_bnnn(nnn),
            (0x0C, _, _, _) => self.op_cxkk(x, kk),
            (0x0D, _, _, _) => self.op_dxyn(x, y, n),
            (0x0E, _, 0x09, 0x0E) => self.op_ex9e(x),
            (0x0E, _, 0x0A, 0x01) => self.op_exa1(x),
            (0x0F, _, 0x00, 0x07) => self.op_fx07(x),
            (0x0F, _, 0x00, 0x0A) => self.op_fx0a(x),
            (0x0F, _, 0x01, 0x05) => self.op_fx15(x),
            (0x0F, _, 0x01, 0x08) => self.op_fx18(x),
            (0x0F, _, 0x01, 0x0E) => self.op_fx1e(x),
            (0x0F, _, 0x02, 0x09) => self.op_fx29(x),
            (0x0F, _, 0x03, 0x03) => self.op_fx33(x),
            (0x0F, _, 0x05, 0x05) => self.op_fx55(x),
            (0x0F, _, 0x06, 0x05) => self.op_fx65(x),
            (_, _, _, _) => {}
        }
    }

    pub fn display_stale(&mut self) -> bool {
        let is_stale = self.display_stale;
        self.display_stale = false;
        return is_stale;
    }
}

impl Chip8 {
    /// CLS: clear the display buffer
    fn op_00e0(&mut self) {
        for i in 0..self.screen.len() {
            for j in 0..self.screen[0].len() {
                self.screen[i][j] = false;
            }
        }
        self.display_stale = true;
    }

    /// RET: return from subroutine
    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    /// JP addr: jump to nnn
    fn op_1nnn(&mut self, nnn: usize) {
        self.pc = nnn as u16;
    }

    /// CALL addr: call subroutine at nnn
    fn op_2nnn(&mut self, nnn: usize) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = nnn as u16;
    }

    /// SE Vx, byte: skip next instruction if Vx == kk
    fn op_3xkk(&mut self, x: usize, kk: u8) {
        if self.v_reg[x] == kk {
            self.pc += 2;
        }
    }

    /// SNE Vx, byte: skip next instruction if Vx != kk
    fn op_4xkk(&mut self, x: usize, kk: u8) {
        if self.v_reg[x] != kk {
            self.pc += 2;
        }
    }

    /// SE Vx, Vy: skip next instruction if Vx == Vy
    fn op_5xy0(&mut self, x: usize, y: usize) {
        if self.v_reg[x] == self.v_reg[y] {
            self.pc += 2;
        }
    }

    /// LD Vx, byte: set Vx = kk
    fn op_6xkk(&mut self, x: usize, kk: u8) {
        self.v_reg[x] = kk;
    }

    /// ADD Vx, byte: add kk to Vx
    fn op_7xkk(&mut self, x: usize, kk: u8) {
        self.v_reg[x] = self.v_reg[x].wrapping_add(kk);
    }

    /// LD Vx, Vy: set Vx = Vy
    fn op_8xy0(&mut self, x: usize, y: usize) {
        self.v_reg[x] = self.v_reg[y];
    }

    /// OR Vx, Vy: set Vx = Vx OR Vy
    fn op_8xy1(&mut self, x: usize, y: usize) {
        self.v_reg[x] |= self.v_reg[y];
    }

    /// AND Vx, Vy: set Vx = Vx AND Vy
    fn op_8xy2(&mut self, x: usize, y: usize) {
        self.v_reg[x] &= self.v_reg[y];
    }

    /// XOR Vx, Vy: set Vx = Vx XOR Vy
    fn op_8xy3(&mut self, x: usize, y: usize) {
        self.v_reg[x] ^= self.v_reg[y];
    }

    /// ADD Vx, Vy: set Vx = Vx + Vy and set VF = carry bit
    fn op_8xy4(&mut self, x: usize, y: usize) {
        let v_x = self.v_reg[x] as u16;
        let v_y = self.v_reg[y] as u16;
        let sum = v_x + v_y;
        self.v_reg[x] = sum as u8;
        self.v_reg[0x0F] = if sum > 0xFF { 1 } else { 0 };
    }

    /// SUB Vx, Vy: set Vx = Vx - Vy and set VF = ~(borrow bit)
    fn op_8xy5(&mut self, x: usize, y: usize) {
        let v_x = self.v_reg[x] as u16;
        let v_y = self.v_reg[y] as u16;
        // let subtract = v_x - v_y;
        let subtract = self.v_reg[x].wrapping_sub(self.v_reg[y]);
        self.v_reg[x] = subtract as u8;
        self.v_reg[0x0F] = if v_x > v_y { 1 } else { 0 };
    }

    /// SUB Vx, Vy: set Vx = Vy - Vx and set VF = ~(borrow bit)
    fn op_8xy7(&mut self, x: usize, y: usize) {
        let v_x = self.v_reg[x] as u16;
        let v_y = self.v_reg[y] as u16;
        // let subtract = v_y - v_x;
        let subtract = self.v_reg[y].wrapping_sub(self.v_reg[x]);
        self.v_reg[x] = subtract as u8;
        self.v_reg[0x0F] = if v_y > v_x { 1 } else { 0 };
    }

    /// SHR Vx: shift Vx one bit right, save shifted-out bit in VF
    fn op_8xy6(&mut self, x: usize, _y: usize) {
        // BELOW ONLY FOR COSMAC VIP INTERPRETER
        // self.v_reg[x] = self.v_reg[y]
        self.v_reg[0x0F] = self.v_reg[x] & 0x01;
        self.v_reg[x] >>= 1;
    }

    /// SHL Vx: shift Vx one bit left, save shifted-out bit in VF
    fn op_8xye(&mut self, x: usize, _y: usize) {
        // BELOW ONLY FOR COSMAC VIP INTERPRETER
        // self.v_reg[x] = self.v_reg[y]
        self.v_reg[0x0F] = self.v_reg[x] & 0x80;
        self.v_reg[x] <<= 1;
    }

    // SNE Vx, Vy: skip if Vx != Vy
    fn op_9xy0(&mut self, x: usize, y: usize) {
        if self.v_reg[x] != self.v_reg[y] {
            self.pc += 2;
        }
    }

    // LD I, addr: load into index register
    fn op_annn(&mut self, nnn: usize) {
        self.i_reg = nnn as u16;
    }

    // JP addr: jump to instruction
    fn op_bnnn(&mut self, nnn: usize) {
        self.pc = (nnn + self.v_reg[0] as usize) as u16;
        // IMPLEMENT CHIP 48 SUPER CHIP QUIRK LATER
    }

    // RND Vx, byte: set Vx = random byte AND kk
    fn op_cxkk(&mut self, x: usize, kk: u8) {
        let mut rand = rand::thread_rng();
        self.v_reg[x] = rand.gen::<u8>() & kk;
    }

    // DRW Vx, Vy, nibble: draw sprite from I at x, y
    fn op_dxyn(&mut self, x: usize, y: usize, n: usize) {
        self.v_reg[0x0F] = 0;
        for row in 0..n {
            let y_coord = (self.v_reg[y] as usize + row) % SCREEN_HEIGHT;
            let sprite = self.ram[self.i_reg as usize + row];
            for shift in 0..8 {
                let pixel = sprite & (0x80 >> shift);
                let x_coord = (self.v_reg[x] as usize + shift) % SCREEN_WIDTH;
                self.v_reg[0x0F] |= pixel & self.screen[y_coord][x_coord] as u8;
                self.screen[y_coord][x_coord] ^= pixel != 0;
            }
        }
        self.display_stale = true;
    }

    // SKP Vx: skip instruction if key in Vx is depressed
    fn op_ex9e(&mut self, x: usize) {
        if self.keypad[self.v_reg[x] as usize] {
            self.pc += 2;
        }
    }

    // SKNP Vx: skip instruction if key in Vx is not depressed
    fn op_exa1(&mut self, x: usize) {
        if !self.keypad[self.v_reg[x] as usize] {
            self.pc += 2;
        }
    }

    // LD Vx, DT: set Vx = delay timer
    fn op_fx07(&mut self, x: usize) {
        self.v_reg[x] = self.delay_timer;
    }

    // LD DT, Vx: set delay timer = Vx
    fn op_fx15(&mut self, x: usize) {
        self.delay_timer = self.v_reg[x];
    }

    // LD ST, Vx: set sound timer = Vx
    fn op_fx18(&mut self, x: usize) {
        self.sound_timer = self.v_reg[x];
    }

    // ADD I, Vx: add Vx to index register
    fn op_fx1e(&mut self, x: usize) {
        self.i_reg += self.v_reg[x] as u16;
        // ADD SUPER-CHIP OVERFLOW BEHAVIOR
    }

    // LD Vx, K: block until key press, store in Vx
    fn op_fx0a(&mut self, x: usize) {
        // NOTE: this is not the correct behavior for COSMAC VIP-style emulation
        // as that system registered keys only when pressed AND released
        for i in 0..self.keypad.len() {
            if self.keypad[i] {
                self.v_reg[x] = i as u8;
                return;
            }
        }
        self.pc -= 2;
    }

    // LD F, Vx: set index register to sprite for char Vx
    fn op_fx29(&mut self, x: usize) {
        self.i_reg = FONTSET_START_ADDRESS + (FONTSET_SPRITE_SIZE * self.v_reg[x] as u16);
    }

    // LD B, Vx: store binary-coded decimal conversion at [I], [I+1], [I+2]
    fn op_fx33(&mut self, x: usize) {
        self.ram[self.i_reg as usize] = self.v_reg[x] / 100;
        self.ram[self.i_reg as usize + 1] = (self.v_reg[x] / 10) % 10;
        self.ram[self.i_reg as usize + 2] = self.v_reg[x] % 10;
    }

    // LD [I], Vx: store registers V0-Vx (inclusive) into memory starting at [I]
    fn op_fx55(&mut self, x: usize) {
        // ADD OLD COSMAC VIP INCREMENTING BEHAVIOR
        for i in 0..=x {
            self.ram[self.i_reg as usize + i] = self.v_reg[i];
        }
    }

    // LD Vx, [I]: load registers V0-Vx (inclusive) from memoery starting at [I]
    fn op_fx65(&mut self, x: usize) {
        // ADD OLD COSMAC VIP INCREMENTING BEHAVIOR
        for i in 0..=x {
            self.v_reg[i] = self.ram[self.i_reg as usize + i];
        }
    }
}

fn main() {
    let mut chippy = Chip8::new();

    let sdl_context = match sdl2::init() {
        Ok(sdl_context) => sdl_context,
        Err(err) => panic!("SDL could not initialize!  SDL_Error: {}", err),
    };

    let mut display = Display::new(&sdl_context);

    let mut input = Input::new(&sdl_context);

    chippy.load("PATH TO FILE");

    while let Ok(keypad) = input.poll() {
        chippy.keypad.copy_from_slice(&keypad);
        chippy.tick();

        if chippy.display_stale() {
            display.draw(&chippy.screen);
        }

        // ensure 500Hz clock rate
        thread::sleep(Duration::from_millis(2));
    }

    for y in 0..SCREEN_HEIGHT {
        for x in 0..SCREEN_WIDTH {
            if chippy.screen[y][x] {
                print!("x");
            } else {
                print!(" ");
            }
        }
        println!();
    }
}
