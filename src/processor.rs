use rand::Rng;
use std::fs::File;
use std::io::Read;

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
pub struct Processor<T: InstructionSet> {
    pub state: ProcessorState,
    isa: T,
}

#[derive(Debug)]
pub struct ProcessorState {
    v_reg: [u8; 16],
    ram: [u8; 4096],
    i_reg: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u16,
    delay_timer: u8,
    sound_timer: u8,
    pub keypad: [bool; 16],
    pub screen: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
    display_stale: bool,
}

#[derive(Debug)]
pub struct SuperChip{}
trait InstructionSet {
    fn execute(&self, cpu: &mut ProcessorState, opcode: u16);
    /// CLS: clear the display buffer
    fn op_00e0(&self, cpu: &mut ProcessorState);
    /// RET: return from subroutine
    fn op_00ee(&self, cpu: &mut ProcessorState);
    /// JP addr: jump to nnn
    fn op_1nnn(&self, cpu: &mut ProcessorState, nnn: usize);
    /// CALL addr: call subroutine at nnn
    fn op_2nnn(&self, cpu: &mut ProcessorState, nnn: usize);
    /// SE Vx, byte: skip next instruction if Vx == kk
    fn op_3xkk(&self, cpu: &mut ProcessorState, x: usize, kk: u8);
    /// SNE Vx, byte: skip next instruction if Vx != kk
    fn op_4xkk(&self, cpu: &mut ProcessorState, x: usize, kk: u8);
    /// SE Vx, Vy: skip next instruction if Vx == Vy
    fn op_5xy0(&self, cpu: &mut ProcessorState, x: usize, y: usize);
    /// LD Vx, byte: set Vx = kk
    fn op_6xkk(&self, cpu: &mut ProcessorState, x: usize, kk: u8);
    /// ADD Vx, byte: add kk to Vx
    fn op_7xkk(&self, cpu: &mut ProcessorState, x: usize, kk: u8);
    /// LD Vx, Vy: set Vx = Vy
    fn op_8xy0(&self, cpu: &mut ProcessorState, x: usize, y: usize);
    /// OR Vx, Vy: set Vx = Vx OR Vy
    fn op_8xy1(&self, cpu: &mut ProcessorState, x: usize, y: usize);
    /// AND Vx, Vy: set Vx = Vx AND Vy
    fn op_8xy2(&self, cpu: &mut ProcessorState, x: usize, y: usize);
    /// XOR Vx, Vy: set Vx = Vx XOR Vy
    fn op_8xy3(&self, cpu: &mut ProcessorState, x: usize, y: usize);
    /// ADD Vx, Vy: set Vx = Vx + Vy and set VF = carry bit
    fn op_8xy4(&self, cpu: &mut ProcessorState, x: usize, y: usize);
    /// SUB Vx, Vy: set Vx = Vx - Vy and set VF = ~(borrow bit)
    fn op_8xy5(&self, cpu: &mut ProcessorState, x: usize, y: usize);
    /// SUB Vx, Vy: set Vx = Vy - Vx and set VF = ~(borrow bit)
    fn op_8xy6(&self, cpu: &mut ProcessorState, x: usize, y: usize);
    /// SHR Vx: shift Vx one bit right, save shifted-out bit in VF
    fn op_8xy7(&self, cpu: &mut ProcessorState, x: usize, y: usize);
    /// SHL Vx: shift Vx one bit left, save shifted-out bit in VF
    fn op_8xye(&self, cpu: &mut ProcessorState, x: usize, y: usize);
    // SNE Vx, Vy: skip if Vx != Vy
    fn op_9xy0(&self, cpu: &mut ProcessorState, x: usize, y: usize);
    // LD I, addr: load into index register
    fn op_annn(&self, cpu: &mut ProcessorState, nnn: usize);
    // JP addr: jump to instruction
    fn op_bnnn(&self, cpu: &mut ProcessorState, nnn: usize);
    // RND Vx, byte: set Vx = random byte AND kk
    fn op_cxkk(&self, cpu: &mut ProcessorState, x: usize, kk: u8);
    // DRW Vx, Vy, nibble: draw sprite from I at x, y
    fn op_dxyn(&self, cpu: &mut ProcessorState, x: usize, y: usize, n: usize);
    // SKP Vx: skip instruction if key in Vx is depressed
    fn op_ex9e(&self, cpu: &mut ProcessorState, x: usize);
    // SKNP Vx: skip instruction if key in Vx is not depressed
    fn op_exa1(&self, cpu: &mut ProcessorState, x: usize);
    // LD Vx, DT: set Vx = delay timer
    fn op_fx07(&self, cpu: &mut ProcessorState, x: usize);
    // LD DT, Vx: set delay timer = Vx
    fn op_fx15(&self, cpu: &mut ProcessorState, x: usize);
    // LD ST, Vx: set sound timer = Vx
    fn op_fx18(&self, cpu: &mut ProcessorState, x: usize);
    // ADD I, Vx: add Vx to index register
    fn op_fx1e(&self, cpu: &mut ProcessorState, x: usize);
    // LD Vx, K: block until key press, store in Vx
    fn op_fx0a(&self, cpu: &mut ProcessorState, x: usize);
    // LD F, Vx: set index register to sprite for char Vx
    fn op_fx29(&self, cpu: &mut ProcessorState, x: usize);
    // LD B, Vx: store binary-coded decimal conversion at [I], [I+1], [I+2]
    fn op_fx33(&self, cpu: &mut ProcessorState, x: usize);
    // LD [I], Vx: store registers V0-Vx (inclusive) into memory starting at [I]
    fn op_fx55(&self, cpu: &mut ProcessorState, x: usize);
    // LD Vx, [I]: load registers V0-Vx (inclusive) from memoery starting at [I]
    fn op_fx65(&self, cpu: &mut ProcessorState, x: usize);
}

impl<T: InstructionSet> Processor<T> {
    pub fn new(isa_variant: T) -> Self {
        let state = ProcessorState {
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

        let mut chip_8 = Self {
            state,
            isa: isa_variant,
        };

        // load fonts into memory
        chip_8.state.ram[0..(FONTSET_START_ADDRESS as usize)].copy_from_slice(&FONT_DATA);

        chip_8
    }

    pub fn load(&mut self, path: &str) -> Result<(), &'static str> {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return Err("Could not open file"),
        };

        let mut rom_buffer = [0u8; 3584];
        file.read(&mut rom_buffer).unwrap_or(0);

        // there's a better way to do this...
        for (i, &byte) in rom_buffer.iter().enumerate() {
            let addr = START_ADDRESS as usize + i;
            if addr < 4096 {
                self.state.ram[addr] = byte;
            } else {
                break;
            }
        }

        Ok(())
    }

    pub fn tick(&mut self) {
        let opcode = self.get_opcode();
        self.state.pc += 2;
        self.isa.execute(&mut self.state, opcode);
        if self.state.delay_timer > 0 {
            self.state.delay_timer -= 1
        }
        if self.state.sound_timer > 0 {
            self.state.sound_timer -= 1
        }
    }

    fn get_opcode(&mut self) -> u16 {
        let high_byte = self.state.ram[self.state.pc as usize] as u16;
        let low_byte = self.state.ram[(self.state.pc + 1) as usize] as u16;
        let opcode = (high_byte << 8) | low_byte;
        opcode
    }

    pub fn display_stale(&mut self) -> bool {
        let is_stale = self.state.display_stale;
        self.state.display_stale = false;
        return is_stale;
    }
}

impl InstructionSet for SuperChip {
    fn execute(&self, cpu: &mut ProcessorState, opcode: u16) {
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
            (0x00, 0, 0xE, 0) => self.op_00e0(cpu),
            (0x00, 0, 0xE, 0xE) => self.op_00ee(cpu,),
            (0x01, _, _, _) => self.op_1nnn(cpu, nnn),
            (0x02, _, _, _) => self.op_2nnn(cpu, nnn),
            (0x03, _, _, _) => self.op_3xkk(cpu, x, kk),
            (0x04, _, _, _) => self.op_4xkk(cpu, x, kk),
            (0x05, _, _, 0x00) => self.op_5xy0(cpu, x, y),
            (0x06, _, _, _) => self.op_6xkk(cpu, x, kk),
            (0x07, _, _, _) => self.op_7xkk(cpu, x, kk),
            (0x08, _, _, 0x00) => self.op_8xy0(cpu, x, y),
            (0x08, _, _, 0x01) => self.op_8xy1(cpu, x, y),
            (0x08, _, _, 0x02) => self.op_8xy2(cpu, x, y),
            (0x08, _, _, 0x03) => self.op_8xy3(cpu, x, y),
            (0x08, _, _, 0x04) => self.op_8xy4(cpu, x, y),
            (0x08, _, _, 0x05) => self.op_8xy5(cpu, x, y),
            (0x08, _, _, 0x06) => self.op_8xy6(cpu, x, y),
            (0x08, _, _, 0x0E) => self.op_8xye(cpu, x, y),
            (0x08, _, _, 0x07) => self.op_8xy7(cpu, x, y),
            (0x09, _, _, _) => self.op_9xy0(cpu, x, y),
            (0x0A, _, _, _) => self.op_annn(cpu, nnn),
            (0x0B, _, _, _) => self.op_bnnn(cpu, nnn),
            (0x0C, _, _, _) => self.op_cxkk(cpu, x, kk),
            (0x0D, _, _, _) => self.op_dxyn(cpu, x, y, n),
            (0x0E, _, 0x09, 0x0E) => self.op_ex9e(cpu, x),
            (0x0E, _, 0x0A, 0x01) => self.op_exa1(cpu, x),
            (0x0F, _, 0x00, 0x07) => self.op_fx07(cpu, x),
            (0x0F, _, 0x00, 0x0A) => self.op_fx0a(cpu, x),
            (0x0F, _, 0x01, 0x05) => self.op_fx15(cpu, x),
            (0x0F, _, 0x01, 0x08) => self.op_fx18(cpu, x),
            (0x0F, _, 0x01, 0x0E) => self.op_fx1e(cpu, x),
            (0x0F, _, 0x02, 0x09) => self.op_fx29(cpu, x),
            (0x0F, _, 0x03, 0x03) => self.op_fx33(cpu, x),
            (0x0F, _, 0x05, 0x05) => self.op_fx55(cpu, x),
            (0x0F, _, 0x06, 0x05) => self.op_fx65(cpu, x),
            (_, _, _, _) => {}
        }
    }

    fn op_00e0(&self, cpu: &mut ProcessorState) {
        for i in 0..cpu.screen.len() {
            for j in 0..cpu.screen[0].len() {
                cpu.screen[i][j] = false;
            }
        }
        cpu.display_stale = true;
    }

    fn op_00ee(&self, cpu: &mut ProcessorState) {
        cpu.sp -= 1;
        cpu.pc = cpu.stack[cpu.sp as usize];
    }

    fn op_1nnn(&self, cpu: &mut ProcessorState, nnn: usize) {
        cpu.pc = nnn as u16;
    }

    fn op_2nnn(&self, cpu: &mut ProcessorState, nnn: usize) {
        cpu.stack[cpu.sp as usize] = cpu.pc;
        cpu.sp += 1;
        cpu.pc = nnn as u16;
    }

    fn op_3xkk(&self, cpu: &mut ProcessorState, x: usize, kk: u8) {
        if cpu.v_reg[x] == kk {
            cpu.pc += 2;
        }
    }

    fn op_4xkk(&self, cpu: &mut ProcessorState, x: usize, kk: u8) {
        if cpu.v_reg[x] != kk {
            cpu.pc += 2;
        }
    }

    fn op_5xy0(&self, cpu: &mut ProcessorState, x: usize, y: usize) {
        if cpu.v_reg[x] == cpu.v_reg[y] {
            cpu.pc += 2;
        }
    }

    fn op_6xkk(&self, cpu: &mut ProcessorState, x: usize, kk: u8) {
        cpu.v_reg[x] = kk;
    }

    fn op_7xkk(&self, cpu: &mut ProcessorState, x: usize, kk: u8) {
        cpu.v_reg[x] = cpu.v_reg[x].wrapping_add(kk);
    }

    fn op_8xy0(&self, cpu: &mut ProcessorState, x: usize, y: usize) {
        cpu.v_reg[x] = cpu.v_reg[y];
    }

    fn op_8xy1(&self, cpu: &mut ProcessorState, x: usize, y: usize) {
        cpu.v_reg[x] |= cpu.v_reg[y];
    }

    fn op_8xy2(&self, cpu: &mut ProcessorState, x: usize, y: usize) {
        cpu.v_reg[x] &= cpu.v_reg[y];
    }

    fn op_8xy3(&self, cpu: &mut ProcessorState, x: usize, y: usize) {
        cpu.v_reg[x] ^= cpu.v_reg[y];
    }

    fn op_8xy4(&self, cpu: &mut ProcessorState, x: usize, y: usize) {
        let v_x = cpu.v_reg[x] as u16;
        let v_y = cpu.v_reg[y] as u16;
        let sum = v_x + v_y;
        cpu.v_reg[x] = sum as u8;
        cpu.v_reg[0x0F] = if sum > 0xFF { 1 } else { 0 };
    }

    fn op_8xy5(&self, cpu: &mut ProcessorState, x: usize, y: usize) {
        let v_x = cpu.v_reg[x] as u16;
        let v_y = cpu.v_reg[y] as u16;
        // let subtract = v_x - v_y;
        let subtract = cpu.v_reg[x].wrapping_sub(cpu.v_reg[y]);
        cpu.v_reg[x] = subtract as u8;
        cpu.v_reg[0x0F] = if v_x > v_y { 1 } else { 0 };
    }

    fn op_8xy7(&self, cpu: &mut ProcessorState, x: usize, y: usize) {
        let v_x = cpu.v_reg[x] as u16;
        let v_y = cpu.v_reg[y] as u16;
        // let subtract = v_y - v_x;
        let subtract = cpu.v_reg[y].wrapping_sub(cpu.v_reg[x]);
        cpu.v_reg[x] = subtract as u8;
        cpu.v_reg[0x0F] = if v_y > v_x { 1 } else { 0 };
    }

    fn op_8xy6(&self, cpu: &mut ProcessorState, x: usize, _y: usize) {
        // BELOW ONLY FOR COSMAC VIP INTERPRETER
        // self.v_reg[x] = self.v_reg[y]
        cpu.v_reg[0x0F] = cpu.v_reg[x] & 0x01;
        cpu.v_reg[x] >>= 1;
    }

    fn op_8xye(&self, cpu: &mut ProcessorState, x: usize, _y: usize) {
        // BELOW ONLY FOR COSMAC VIP INTERPRETER
        // self.v_reg[x] = self.v_reg[y]
        cpu.v_reg[0x0F] = cpu.v_reg[x] & 0x80;
        cpu.v_reg[x] <<= 1;
    }

    fn op_9xy0(&self, cpu: &mut ProcessorState, x: usize, y: usize) {
        if cpu.v_reg[x] != cpu.v_reg[y] {
            cpu.pc += 2;
        }
    }

    fn op_annn(&self, cpu: &mut ProcessorState, nnn: usize) {
        cpu.i_reg = nnn as u16;
    }

    fn op_bnnn(&self, cpu: &mut ProcessorState, nnn: usize) {
        cpu.pc = (nnn + cpu.v_reg[0] as usize) as u16;
        // IMPLEMENT CHIP 48 SUPER CHIP QUIRK LATER
    }

    fn op_cxkk(&self, cpu: &mut ProcessorState, x: usize, kk: u8) {
        let mut rand = rand::thread_rng();
        cpu.v_reg[x] = rand.gen::<u8>() & kk;
    }

    fn op_dxyn(&self, cpu: &mut ProcessorState, x: usize, y: usize, n: usize) {
        cpu.v_reg[0x0F] = 0;
        for row in 0..n {
            let y_coord = (cpu.v_reg[y] as usize + row) % SCREEN_HEIGHT;
            let sprite = cpu.ram[cpu.i_reg as usize + row];
            for shift in 0..8 {
                let pixel = sprite & (0x80 >> shift);
                let x_coord = (cpu.v_reg[x] as usize + shift) % SCREEN_WIDTH;
                cpu.v_reg[0x0F] |= pixel & cpu.screen[y_coord][x_coord] as u8;
                cpu.screen[y_coord][x_coord] ^= pixel != 0;
            }
        }
        cpu.display_stale = true;
    }

    fn op_ex9e(&self, cpu: &mut ProcessorState, x: usize) {
        if cpu.keypad[cpu.v_reg[x] as usize] {
            cpu.pc += 2;
        }
    }

    fn op_exa1(&self, cpu: &mut ProcessorState, x: usize) {
        if !cpu.keypad[cpu.v_reg[x] as usize] {
            cpu.pc += 2;
        }
    }

    fn op_fx07(&self, cpu: &mut ProcessorState, x: usize) {
        cpu.v_reg[x] = cpu.delay_timer;
    }

    fn op_fx15(&self, cpu: &mut ProcessorState, x: usize) {
        cpu.delay_timer = cpu.v_reg[x];
    }

    fn op_fx18(&self, cpu: &mut ProcessorState, x: usize) {
        cpu.sound_timer = cpu.v_reg[x];
    }

    fn op_fx1e(&self, cpu: &mut ProcessorState, x: usize) {
        cpu.i_reg += cpu.v_reg[x] as u16;
        // ADD SUPER-CHIP OVERFLOW BEHAVIOR
    }

    fn op_fx0a(&self, cpu: &mut ProcessorState, x: usize) {
        // NOTE: this is not the correct behavior for COSMAC VIP-style emulation
        // as that system registered keys only when pressed AND released
        for i in 0..cpu.keypad.len() {
            if cpu.keypad[i] {
                cpu.v_reg[x] = i as u8;
                return;
            }
        }
        cpu.pc -= 2;
    }

    fn op_fx29(&self, cpu: &mut ProcessorState, x: usize) {
        cpu.i_reg = FONTSET_START_ADDRESS + (FONTSET_SPRITE_SIZE * cpu.v_reg[x] as u16);
    }

    fn op_fx33(&self, cpu: &mut ProcessorState, x: usize) {
        cpu.ram[cpu.i_reg as usize] = cpu.v_reg[x] / 100;
        cpu.ram[cpu.i_reg as usize + 1] = (cpu.v_reg[x] / 10) % 10;
        cpu.ram[cpu.i_reg as usize + 2] = cpu.v_reg[x] % 10;
    }

    fn op_fx55(&self, cpu: &mut ProcessorState, x: usize) {
        // ADD OLD COSMAC VIP INCREMENTING BEHAVIOR
        for i in 0..=x {
            cpu.ram[cpu.i_reg as usize + i] = cpu.v_reg[i];
        }
    }

    fn op_fx65(&self, cpu: &mut ProcessorState, x: usize) {
        // ADD OLD COSMAC VIP INCREMENTING BEHAVIOR
        for i in 0..=x {
            cpu.v_reg[i] = cpu.ram[cpu.i_reg as usize + i];
        }
    }
}