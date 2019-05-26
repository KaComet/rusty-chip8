//! Interpreter for original chip-8 programs.

extern crate rand;
use rand::Rng;

pub struct Chip8
{
    device_state:       CpuState,
    tick_delay:         u16,
    opcode:             u8,
    index:              u16,
    program_counter:    u16,
    timer_delay:        f32,
    buzzer_delay:       f32,
    stack_pointer:      u8,
    temp_vx:            u8,
    stack:             [u16; 16],
    general_registers: [u8; 16],
    memory:            [u8; 4096],
    keypad:            [KeyState; 16],
    temp_keypad:       [KeyState; 16],
    screen:            [PixelState; 64 * 32]
}

impl Default for Chip8 
{
    //Returns a chip8 struct with some default values.
    fn default() -> Self { 
        Chip8
        {
            device_state:          CpuState::Ready,
            tick_delay:         0,
            opcode:             0,
            index:              0,
            program_counter:    0,
            timer_delay:        0.0,
            buzzer_delay:       0.0,
            stack_pointer:      0,
            temp_vx:            0,
            stack:             [0; 16],
            general_registers: [0; 16],
            memory:            [0; 4096],
            keypad:            [KeyState::Unpressed; 16],
            temp_keypad:       [KeyState::Unpressed; 16],
            screen:            [PixelState::Unlit; 64 * 32]
        } 
    }
}

#[derive(PartialEq)]
#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum KeyState
{
    Pressed,
    Unpressed
}

#[derive(PartialEq)]
#[derive(Clone, Copy)]
pub enum PixelState
{
    Lit,
    Unlit
}

#[derive(PartialEq)]
#[derive(Clone, Copy)]
pub enum CpuState
{
    Ready,
    WaitingForKeypress
}

impl Chip8
{
    ///! Performs a soft reset. (clears all registers and sets the PC to 0x200)
    pub fn soft_reset(&mut self)
    {
        self.opcode          = 0x000;
        self.index           = 0x000;
        self.program_counter = 0x200;
        self.timer_delay     = 0.000;
        self.buzzer_delay    = 0.000;
        self.stack_pointer   = 0x000;
        self.device_state    = CpuState::Ready;

        return;
    }

    ///! Performs a hard reset. (resets the device's registers, memory, screen, keyboard, and font data)
    pub fn hard_reset(&mut self)
    {
        self.soft_reset();
        for i in 0..15        {self.stack[i]             = 0x00}
        for i in 0..15        {self.general_registers[i] = 0x00}
        for i in 0..4096      {self.memory[i]            = 0x00}
        for i in 0..16        {self.keypad[i]            = KeyState::Unpressed}
        for i in 0..16        {self.temp_keypad[i]       = KeyState::Unpressed}
        for i in 0..(64 * 32) {self.screen[i]            = PixelState::Unlit}

        //Because the device's entire memory has been cleard, we must reload the default fontset.
        self.load_default_font();

        return;
    }

    fn save_keypad(&mut self)
    {
        for i in 0..16
        {
            self.temp_keypad[i] = self.keypad[i];
        }

        return
    }

    fn check_for_new_key_pressed(&mut self)
    {
        for i in 0..16
        {
            if (self.temp_keypad[i] == KeyState::Unpressed) && (self.keypad[i] == KeyState::Pressed)
            {
                self.device_state = CpuState::Ready;
                self.opcode_LD_VX_K_CONT(self.temp_vx, i as u8);

                break;
            }
        }

        return;
    }

    ///! For setting a single byte of the device's memory. (don't forget to include a 0x200 byte offset for program data)
    pub fn set_memory_byte(&mut self, address: u16, byte: u8) -> bool
    {
        if address < 4096
        {
            self.memory[address as usize] = byte;
            true
        }
        else
        {
            false
        }
    }

    ///! For setting a single word (2 bytes) of the device's memory. (don't forget to include a 0x200 byte offset for program data)
    pub fn set_memory_word(&mut self, address: u16, word: u16) -> bool
    {
        if address < 4096
        {
            self.memory[address as usize] = (word >> 8) as u8;
            self.memory[(address + 1) as usize] = (word & 0xFF) as u8;
            true
        }
        else
        {
            false
        }
    }

    ///! Sets the devices key to the desired state.
    pub fn set_key(&mut self, key_number: u8, desired_state: KeyState) -> bool
    {
        if key_number < 16
        {
            self.keypad[key_number as usize] = desired_state;
            true
        }
        else
        {
            false
        }
    }

    ///! Returns the state of the pixel at the indicated row and column.
    pub fn get_screen_pixel(&mut self, row: u8, col: u8) -> PixelState
    {
        if (row  < 32) && (col < 64)
        {
            let pixel: u16 = (64 * row as u16) + col as u16;
            match self.screen[pixel as usize]
            {
                PixelState::Lit   => PixelState::Lit,
                PixelState::Unlit => PixelState::Unlit
            }
        }
        else 
        {
            PixelState::Unlit
        }
    }

    ///! Set the pixel to the desired state at the indicated row and column.
    pub fn set_screen_pixel(&mut self, row: u8, col: u8, desired_state: PixelState)
    {
        if (row  < 32) && (col < 64)
        {
            self.screen[((64 * row as u16) + col as u16) as usize] = desired_state;
        }

        return;
    }

    ///! Sets the program counter to the indicated byte.
    pub fn set_program_counter(&mut self, desired_pc_value: u16)
    {
        if desired_pc_value >= 4096
        {
            return
        }

        self.program_counter = desired_pc_value;
        
        return;
    }

    //Subtracts the indicated value from the delay counter.
    pub fn subtract_from_delaycounter(&mut self, value_to_subtract: f32)
    {
        self.timer_delay -= value_to_subtract;
        if self.timer_delay < 0.0
        {
            self.timer_delay = 0.0;
        }

        return
    }

    pub fn subtract_from_buzzercounter(&mut self, value_to_subtract: f32)
    {
        self.buzzer_delay -= value_to_subtract;
        if self.buzzer_delay < 0.0
        {
            self.buzzer_delay = 0.0;
        }

        return
    }

    fn load_default_font(&mut self)
    {
        let font_set: [u8; 80] = 
        [
            0xF0, 0x90, 0x90, 0x90, 0xF0,		// 0
	        0x20, 0x60, 0x20, 0x20, 0x70,		// 1
	        0xF0, 0x10, 0xF0, 0x80, 0xF0,		// 2
	        0xF0, 0x10, 0xF0, 0x10, 0xF0,		// 3
    	    0x90, 0x90, 0xF0, 0x10, 0x10,		// 4
    	    0xF0, 0x80, 0xF0, 0x10, 0xF0,		// 5
    	    0xF0, 0x80, 0xF0, 0x90, 0xF0,		// 6
    	    0xF0, 0x10, 0x20, 0x40, 0x40,		// 7
    	    0xF0, 0x90, 0xF0, 0x90, 0xF0,		// 8
    	    0xF0, 0x90, 0xF0, 0x10, 0xF0,		// 9
	        0xF0, 0x90, 0xF0, 0x90, 0x90,		// A
    	    0xE0, 0x90, 0xE0, 0x90, 0xE0,		// B
    	    0xF0, 0x80, 0x80, 0x80, 0xF0,		// C
    	    0xE0, 0x90, 0x90, 0x90, 0xE0,		// D
    	    0xF0, 0x80, 0xF0, 0x80, 0xF0,		// E
    	    0xF0, 0x80, 0xF0, 0x80, 0x80		// F
        ];

        //Load copy fontset into the devices memory
        for i in 0..80
        {
            self.set_memory_byte(i, font_set[i as usize]);
        }
    }

    ///! Fully executes one instruction. Automatically increments the program counter as needed.
    pub fn execute(&mut self)
    {
        match self.device_state
        {
            CpuState::WaitingForKeypress => { self.check_for_new_key_pressed(); },
            CpuState::Ready                => {
                //Split the 16-byte opcode into four 4-bit nibbles. This will allow us to use pattern matching to detect the opcode.
                let nibble3: u8 = (self.memory[ self.program_counter as usize]      & 0xF0) >> 4;
                let nibble2: u8 =  self.memory[ self.program_counter as usize]      & 0x0F;
                let nibble1: u8 = (self.memory[(self.program_counter as usize) + 1] & 0xF0) >> 4;
                let nibble0: u8 =  self.memory[(self.program_counter as usize) + 1] & 0x0F;

                //Decode the current instruction then execute the instruction.
                match (nibble3, nibble2, nibble1, nibble0)
                {
                    (0x0, 0x0, 0xE, 0x0) => self.opcode_CLS       (), //t
                    (0x0, 0x0, 0xE, 0xE) => self.opcode_RET       (), //t
                    (0x0,   _,   _,   _) => self.opcode_SYS       (), //t
                    (0x1,   _,   _,   _) => self.opcode_JP        (nibble2, (nibble1 << 4) | nibble0), //t
                    (0x2,   _,   _,   _) => self.opcode_CALL      (nibble2, (nibble1 << 4) | nibble0), //t
                    (0x3,   _,   _,   _) => self.opcode_SE_VX     (nibble2, (nibble1 << 4) | nibble0), //t
                    (0x4,   _,   _,   _) => self.opcode_SNE_VX    (nibble2, (nibble1 << 4) | nibble0), //t
                    (0x5,   _,   _, 0x0) => self.opcode_SE_VX_VY  (nibble2, nibble1), //t
                    (0x6,   _,   _,   _) => self.opcode_LD_VX     (nibble2, (nibble1 << 4) | nibble0), //t
                    (0x7,   _,   _,   _) => self.opcode_ADD_VX    (nibble2, (nibble1 << 4) | nibble0), //t
                    (0x8,   _,   _, 0x0) => self.opcode_LD_VX_VY  (nibble2, nibble1), //t
                    (0x8,   _,   _, 0x1) => self.opcode_OR_VX_VY  (nibble2, nibble1), //t
                    (0x8,   _,   _, 0x2) => self.opcode_AND_VX_VY (nibble2, nibble1), //t
                    (0x8,   _,   _, 0x3) => self.opcode_XOR_VX_VY (nibble2, nibble1), //t
                    (0x8,   _,   _, 0x4) => self.opcode_ADD_VX_VY (nibble2, nibble1), //t
                    (0x8,   _,   _, 0x5) => self.opcode_SUB_VX_VY (nibble2, nibble1), //t
                    (0x8,   _,   _, 0x6) => self.opcode_SHR_VX    (nibble2),          //t
                    (0x8,   _,   _, 0x7) => self.opcode_SUBN_VX_VY(nibble2, nibble1), //t
                    (0x8,   _,   _, 0xE) => self.opcode_SHL_VX    (nibble2),          //t
                    (0x9,   _,   _, 0x0) => self.opcode_SNE_VX_VY (nibble2, nibble1), //t
                    (0xA,   _,   _,   _) => self.opcode_LD_I      (nibble2, (nibble1 << 4) | nibble0), //t
                    (0xB,   _,   _,   _) => self.opcode_JP_V0     (nibble2, (nibble1 << 4) | nibble0), //t
                    (0xC,   _,   _,   _) => self.opcode_RND_VX    (nibble2, (nibble1 << 4) | nibble0), //Not working as intended
                    (0xD,   _,   _,   _) => self.opcode_DRW_VX_VY (nibble2, nibble1, nibble0), //t
                    (0xE,   _, 0x9, 0xE) => self.opcode_SKP_VX    (nibble2), //t
                    (0xE,   _, 0xA, 0x1) => self.opcode_SKNP_VX   (nibble2),
                    (0xF,   _, 0x0, 0x7) => self.opcode_LD_VX_DT  (nibble2),
                    (0xF,   _, 0x0, 0xA) => self.opcode_LD_VX_K   (nibble2), //t
                    (0xF,   _, 0x1, 0x5) => self.opcode_LD_DT_VX  (nibble2),
                    (0xF,   _, 0x1, 0x8) => self.opcode_LD_ST_VX  (nibble2),
                    (0xF,   _, 0x1, 0xE) => self.opcode_ADD_I_VX  (nibble2), //t
                    (0xF,   _, 0x2, 0x9) => self.opcode_LD_F_VX   (nibble2), //t
                    (0xF,   _, 0x3, 0x3) => self.opcode_LD_B_VX   (nibble2),
                    (0xF,   _, 0x5, 0x5) => self.opcode_LD_iIi_VX (nibble2),
                    (0xF,   _, 0x6, 0x5) => self.opcode_LD_VX_iIi (nibble2),
                    (  _,   _,   _,   _) => ()
                }

                if self.program_counter >= 4096
                {
                    self.program_counter %= 4096;
                }
            }
        }
    }


    //Function for execution of CLS opcode. Clears the screen.
    #[allow(non_snake_case)]
    fn opcode_CLS(&mut self)
    {
        for i in 0..(64 * 32) 
        {
            self.screen[i] = PixelState::Unlit
        }
        self.program_counter += 2;
        self.tick_delay += 1;

        return;
    }

    #[allow(non_snake_case)]
    fn opcode_SYS(&mut self)
    {
        self.program_counter += 2;
        self.tick_delay += 1;

        return;
    }

    #[allow(non_snake_case)]
    fn opcode_RET(&mut self)
    {
        //println!("Boing!");
        self.program_counter = self.stack[self.stack_pointer as usize] + 2;
        if self.stack_pointer == 0
        {
            self.stack_pointer = 0xF;
        }
        else
        {
            self.stack_pointer -= 1;
        }

        if self.program_counter >= 4096
        {
            self.program_counter %= 4096;
        }

        self.tick_delay += 1;

        return;
    }

    #[allow(non_snake_case)]
    fn opcode_JP(&mut self, n: u8, nn: u8)
    {
        self.program_counter = ((n as u16) << 8) | (nn as u16);  
        self.tick_delay += 1;

        return;
    }

    #[allow(non_snake_case)]
    fn opcode_CALL(&mut self, n: u8, nn: u8)
    {
        //println!("JP: {} \t{}", n, nn);
        self.stack_pointer += 1;
        if self.stack_pointer > 0xF
        {
            self.stack_pointer = 0;
        }
        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.program_counter = ((n as u16) << 8) | (nn as u16);
        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_SE_VX(&mut self, vx: u8, kk: u8)
    {
        if kk == self.general_registers[vx as usize]
        {
            self.program_counter += 4; // Skips next instruction
        }
        else
        {
            self.program_counter += 2;
        }
        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_SNE_VX(&mut self, vx: u8, kk: u8)
    {
        if kk != self.general_registers[vx as usize]
        {
            self.program_counter += 4; // Skips next instruction
        }
        else
        {
            self.program_counter += 2;
        }
        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_SE_VX_VY(&mut self, vx: u8, vy: u8)
    {
        if self.general_registers[vx as usize] == self.general_registers[vy as usize]
        {
            self.program_counter += 4; // Skips next instruction
        }
        else
        {
            self.program_counter += 2;
        }
        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_LD_VX(&mut self, vx: u8, kk: u8)
    {
        self.general_registers[vx as usize] = kk;
        self.program_counter += 2;
        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_ADD_VX(&mut self, vx: u8, kk: u8)
    {
        self.general_registers[vx as usize] = self.general_registers[vx as usize].wrapping_add(kk);
        self.program_counter += 2;
        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_LD_VX_VY(&mut self, vx: u8, vy: u8)
    {
        self.general_registers[vx as usize] = self.general_registers[vy as usize];
        self.program_counter += 2;
        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_OR_VX_VY(&mut self, vx: u8, vy: u8)
    {
        self.general_registers[vx as usize] |= self.general_registers[vy as usize];
        self.program_counter += 2;
        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_AND_VX_VY(&mut self, vx: u8, vy: u8)
    {
        self.general_registers[vx as usize] &= self.general_registers[vy as usize];
        self.program_counter += 2;
        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_XOR_VX_VY(&mut self, vx: u8, vy: u8)
    {
        self.general_registers[vx as usize] ^= self.general_registers[vy as usize];
        self.program_counter += 2;
        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_ADD_VX_VY(&mut self, vx: u8, vy: u8)
    {
        if (self.general_registers[vx as usize] as u16) + (self.general_registers[vy as usize] as u16) > 0xFF
        {
            self.general_registers[0xF] = 1;
        }
        else
        {
            self.general_registers[0xF] = 0;
        }
        self.general_registers[vx as usize] = self.general_registers[vx as usize].wrapping_sub(self.general_registers[vy as usize]);
        
        self.program_counter += 2;
        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_SUB_VX_VY(&mut self, vx: u8, vy: u8)
    {
        if (self.general_registers[vx as usize] as i16) - (self.general_registers[vy as usize] as i16) < 0
        {
            self.general_registers[0xF] = 0;
        }
        else
        {
            self.general_registers[0xF] = 1;
        }

        self.general_registers[vx as usize] = self.general_registers[vx as usize].wrapping_sub(self.general_registers[vy as usize]);
        
        self.program_counter += 2;
        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_SHR_VX(&mut self, vx: u8)
    {
        if (self.general_registers[vx as usize] & 1) != 0
        {
            self.general_registers[0xF] = 1;
        }
        else
        {
            self.general_registers[0xF] = 0;
        }
        self.general_registers[vx as usize] >>= 1;
        
        self.program_counter += 2;
        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_SUBN_VX_VY(&mut self, vx: u8, vy: u8)
    {
        if (self.general_registers[vy as usize] as i16) - (self.general_registers[vx as usize] as i16) < 0
        {
            self.general_registers[0xF] = 0;
        }
        else
        {
            self.general_registers[0xF] = 1;
        }
        self.general_registers[vx as usize] = self.general_registers[vx as usize].wrapping_sub(self.general_registers[vy as usize]);
        
        self.program_counter += 2;
        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_SHL_VX(&mut self, vx: u8)
    {
        if (self.general_registers[vx as usize] & 0b10000000) == 0
        {
            self.general_registers[0xF] = 0;
        }
        else
        {
            self.general_registers[0xF] = 1;
        }

        self.general_registers[vx as usize] <<= 2;

        self.program_counter += 2;
        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_SNE_VX_VY(&mut self, vx: u8, vy: u8)
    {
        if self.general_registers[vx as usize] != self.general_registers[vy as usize]
        {
            self.program_counter += 4; // Skips next instruction
        }
        else
        {
            self.program_counter += 2;
        }
        self.tick_delay += 1;

        return;
    }

    #[allow(non_snake_case)]
    fn opcode_LD_I(&mut self, n: u8, nn: u8)
    {
        self.index = ((n as u16) << 8) | (nn as u16);

        self.program_counter += 2;
        self.tick_delay += 1;

        return;
    }

    #[allow(non_snake_case)]
    fn opcode_JP_V0(&mut self, n: u8, nn: u8)
    {
        self.program_counter = (((n as u16) << 8) | (nn as u16)) + (self.general_registers[0] as u16);

        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_RND_VX(&mut self, vx: u8, kk: u8)
    {
        let mut rng = rand::thread_rng();
        self.general_registers[vx as usize] = kk & rng.gen::<u8>();

        self.program_counter += 2;
        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for general_registers, index, memory, sprite wrapping
    #[allow(non_snake_case)]
    fn opcode_DRW_VX_VY(&mut self, vx: u8, vy: u8, n: u8)
    {
        let vx = vx % 64;
        let vy = vy % 32;
        let x_pos = self.general_registers[vx as usize];
        let y_pos = self.general_registers[vy as usize];
        self.general_registers[0xF] = 0;

        for current_sprite_pixel_y in 0..n
        {
            for current_sprite_pixel_x in 0..8
            {
                let mut screen_pixel_x = (current_sprite_pixel_x as u16) + (x_pos as u16);
                let mut screen_pixel_y = (current_sprite_pixel_y as u16) + (y_pos as u16);

                if screen_pixel_x > 63
                {
                    screen_pixel_x = screen_pixel_x % 64;
                }

                if screen_pixel_y > 31
                {
                    screen_pixel_y = screen_pixel_y % 32;
                }

                let mut pixel: PixelState = PixelState::Unlit;
                let pixel_bit: u8 = (0b10000000 >> current_sprite_pixel_x) & self.memory[((self.index as u16) + (current_sprite_pixel_y as u16)) as usize];
                
                if pixel_bit != 0
                {
                    pixel = PixelState::Lit;
                }

                let current_pixel: PixelState = self.screen[((screen_pixel_x as u16) + ((screen_pixel_y as u16) * 64)) as usize];
                match (current_pixel, pixel)
                {
                    (PixelState::Unlit, PixelState::Lit)   =>  pixel = PixelState::Lit,
                    (PixelState::Lit,   PixelState::Lit)   => {pixel = PixelState::Unlit; self.general_registers[0xF] = 1;},
                    (PixelState::Unlit, PixelState::Unlit) =>  pixel = PixelState::Unlit,
                    (PixelState::Lit,   PixelState::Unlit) =>  pixel = PixelState::Lit
                }

                self.screen[((screen_pixel_x as u16) + ((screen_pixel_y as u16) * 64)) as usize] = pixel;
            }
        }

        self.program_counter += 2;
        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for keypad
    #[allow(non_snake_case)]
    fn opcode_SKP_VX(&mut self, vx: u8)
    {
        if vx > 0xF
        {
            self.program_counter += 2;
            return;
        }

        if self.keypad[self.general_registers[vx as usize] as usize] == KeyState::Pressed
        {
            self.program_counter += 4;
        }
        else
        {
            self.program_counter += 2;
        }

        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for keypad
    #[allow(non_snake_case)]
    fn opcode_SKNP_VX(&mut self, vx: u8)
    {
        if vx > 0xF
        {
            self.program_counter += 2;
            return;
        }

        if self.keypad[vx as usize] == KeyState::Unpressed
        {
            self.program_counter += 4;
        }
        else
        {
            self.program_counter += 2;
        }

        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for keypad
    #[allow(non_snake_case)]
    fn opcode_LD_VX_DT(&mut self, vx: u8)
    {
        self.general_registers[vx as usize] = self.timer_delay.ceil() as u8;

        self.program_counter += 2;
        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_LD_VX_K(&mut self, vx: u8)
    {
        self.temp_vx = vx;
        self.device_state = CpuState::WaitingForKeypress;
        self.save_keypad();

        return;
    }

    //Executed after a keypress is performed
    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_LD_VX_K_CONT(&mut self, vx: u8, pressed_key: u8)
    {
        self.general_registers[vx as usize] = pressed_key;
        //println!("reg: {}     key: {}", vx, pressed_key);
        self.program_counter += 2;
        self.tick_delay += 1;

        if self.program_counter >= 4096
        {
            self.program_counter %= 4096;
        }

        return;
    }

    //TODO: bounds check for keypad
    #[allow(non_snake_case)]
    fn opcode_LD_DT_VX(&mut self, vx: u8)
    {
        self.timer_delay = self.general_registers[vx as usize] as f32;

        self.program_counter += 2;
        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for keypad
    #[allow(non_snake_case)]
    fn opcode_LD_ST_VX(&mut self, vx: u8)
    {
        self.buzzer_delay = self.general_registers[vx as usize] as f32;

        self.program_counter += 2;
        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_ADD_I_VX(&mut self, vx: u8)
    {
        self.index = self.index.wrapping_add(self.general_registers[vx as usize] as u16);

        self.program_counter += 2;
        self.tick_delay += 1;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_LD_F_VX(&mut self, vx: u8)
    {
        self.index = 5 * (self.general_registers[vx as usize] as u16);

        self.program_counter += 2;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_LD_B_VX(&mut self, vx: u8)
    {
        let value: u8    = self.general_registers[vx as usize];
        let mut result: [u8; 3] = [0,0,0];
        result[0] = value / 100;
        result[1] = (value - result[0]) / 10;
        result[2] = value - (result[0] + result[1]);

        for i in 0..3
        {
            if (self.index + i) >= 4096
            {
                break;
            }

            self.memory[(self.index + i) as usize] = result[i as usize];
        }

        self.program_counter += 2;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_LD_iIi_VX(&mut self, vx: u8)
    {
        for register_number in 0..=vx
        {
            if (self.index + (register_number as u16) >= 4096) || (vx > 0xF)
            {
                break;
            }

            self.memory[(self.index + (register_number as u16)) as usize] = self.general_registers[register_number as usize];
        }

        self.program_counter += 2;

        return;
    }

    //TODO: bounds check for general_registers
    #[allow(non_snake_case)]
    fn opcode_LD_VX_iIi(&mut self, vx: u8)
    {
        for register_number in 0..=vx
        {
            if ((self.index + (register_number as u16)) >= 4096) || (vx > 0xF)
            {
                break;
            }

            self.general_registers[register_number as usize] = self.memory[(self.index + (register_number as u16)) as usize];
        }

        self.program_counter += 2;

        return;
    }

}