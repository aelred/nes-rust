use super::addressing_modes::BITAddressingMode;
use super::addressing_modes::CompareAddressingMode;
use super::addressing_modes::FlexibleAddressingMode;
use super::addressing_modes::IncDecAddressingMode;
use super::addressing_modes::JumpAddressingMode;
use super::addressing_modes::LDXAddressingMode;
use super::addressing_modes::LDYAddressingMode;
use super::addressing_modes::ShiftAddressingMode;
use super::addressing_modes::StoreAddressingMode;
use super::addressing_modes::STXAddressingMode;
use super::addressing_modes::STYAddressingMode;

#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    // Load / Store Operations
    /// Load Accumulator
    ///
    /// A,Z,N = M
    ///
    /// Loads a byte of memory into the accumulator setting the zero and negative flags as
    /// appropriate.
    LDA(FlexibleAddressingMode),

    /// Load X Register
    ///
    /// X,Z,N = M
    ///
    /// Loads a byte of memory into the X register setting the zero and negative flags as
    /// appropriate.
    LDX(LDXAddressingMode),

    /// Load Y Register
    ///
    /// Y,Z,N = M
    ///
    /// Loads a byte of memory into the Y register setting the zero and negative flags as
    /// appropriate.
    LDY(LDYAddressingMode),

    /// Store Accumulator
    ///
    /// M = A
    ///
    /// Stores the contents of the accumulator into memory.
    STA(StoreAddressingMode),

    /// Store X Register
    ///
    /// M = X
    ///
    /// Stores the contents of the X register into memory.
    STX(STXAddressingMode),

    /// Store Y Register
    ///
    /// M = Y
    ///
    /// Stores the contents of the Y register into memory.
    STY(STYAddressingMode),

    // Register Transfers
    /// Transfer Accumulator to X
    ///
    /// X = A
    //
    /// Copies the current contents of the accumulator into the X register and sets the zero and
    /// negative flags as appropriate.
    TAX,

    /// Transfer Accumulator to Y
    ///
    /// Y = A
    ///
    /// Copies the current contents of the accumulator into the Y register and sets the zero and
    /// negative flags as appropriate.
    TAY,

    /// Transfer X to Accumulator
    ///
    /// A = X
    ///
    /// Copies the current contents of the X register into the accumulator and sets the zero and
    /// negative flags as appropriate.
    TXA,

    /// Transfer Y to Accumulator
    ///
    /// A = Y
    ///
    /// Copies the current contents of the Y register into the accumulator and sets the zero and
    /// negative flags as appropriate.
    TYA,

    // Stack Operations
    /// Transfer Stack Pointer to X
    ///
    /// X = S
    ///
    /// Copies the current contents of the stack register into the X register and sets the zero and
    /// negative flags as appropriate.
    TSX,

    /// Transfer X to Stack Pointer
    ///
    /// S = X
    ///
    /// Copies the current contents of the X register into the stack register.
    TXS,

    /// Push Accumulator
    ///
    /// Pushes a copy of the accumulator on to the stack.
    PHA,

    /// Push Processor Status
    ///
    /// Pushes a copy of the status flags on to the stack.
    PHP,

    /// Pull Accumulator
    ///
    /// Pulls an 8 bit value from the stack and into the accumulator. The zero and negative flags
    /// are set as appropriate.
    PLA,

    /// Pull Processor Status
    ///
    /// Pulls an 8 bit value from the stack and into the processor flags. The flags will take on new
    /// states as determined by the value pulled.
    PLP,

    /// Logical AND
    ///
    /// A,Z,N = A&M
    ///
    /// A logical AND is performed, bit by bit, on the accumulator contents using the contents of a
    /// byte of memory.
    AND(FlexibleAddressingMode),

    /// Exclusive OR
    ///
    /// A,Z,N = A^M
    ///
    /// An exclusive OR is performed, bit by bit, on the accumulator contents using the contents of
    /// a byte of memory.
    EOR(FlexibleAddressingMode),

    /// Logical Inclusive OR
    ///
    /// A,Z,N = A|M
    ///
    /// An inclusive OR is performed, bit by bit, on the accumulator contents using the contents of
    /// a byte of memory.
    ORA(FlexibleAddressingMode),

    /// Bit Test
    ///
    /// A & M, N = M7, V = M6
    ///
    /// This instructions is used to test if one or more bits are set in a target memory location.
    /// The mask pattern in A is ANDed with the value in memory to set or clear the zero flag, but
    /// the result is not kept. Bits 7 and 6 of the value from memory are copied into the N and V
    /// flags.
    BIT(BITAddressingMode),

    // Arithmetic
    /// Add With Carry
    ///
    /// A,Z,C,N = A+M+C
    ///
    /// This instruction adds the contents of a memory location to the accumulator together with the
    /// carry bit. If overflow occurs the carry bit is set, this enables multiple byte addition to
    /// be performed.
    ADC(FlexibleAddressingMode),

    /// Subtract with Carry
    ///
    /// A,Z,C,N = A-M-(1-C)
    ///
    /// This instruction subtracts the contents of a memory location to the accumulator together
    /// with the not of the carry bit. If overflow occurs the carry bit is clear, this enables
    /// multiple byte subtraction to be performed.
    SBC(FlexibleAddressingMode),

    /// Compare
    ///
    /// Z,C,N = A-M
    ///
    /// This instruction compares the contents of the accumulator with another memory held value and
    /// sets the zero and carry flags as appropriate.
    CMP(FlexibleAddressingMode),

    /// Compare X Register
    ///
    /// Z,C,N = X-M
    ///
    /// This instruction compares the contents of the X register with another memory held value and
    /// sets the zero and carry flags as appropriate.
    CPX(CompareAddressingMode),

    /// Compare Y Register
    ///
    /// Z,C,N = Y-M
    ///
    /// This instruction compares the contents of the Y register with another memory held value and
    /// sets the zero and carry flags as appropriate.
    CPY(CompareAddressingMode),

    // Increments & Decrements
    /// Increment Memory
    ///
    /// M,Z,N = M+1
    ///
    /// Adds one to the value held at a specified memory location setting the zero and negative
    /// flags as appropriate.
    INC(IncDecAddressingMode),

    /// Increment X Register
    ///
    /// X,Z,N = X+1
    ///
    /// Adds one to the X register setting the zero and negative flags as appropriate.
    INX,

    /// Increment Y Register
    /// Y,Z,N = Y+1
    ///
    /// Adds one to the Y register setting the zero and negative flags as appropriate.
    INY,

    /// Decrement Memory
    ///
    /// M,Z,N = M-1
    ///
    /// Subtracts one from the value held at a specified memory location setting the zero and
    /// negative flags as appropriate.
    DEC(IncDecAddressingMode),

    /// Decrement X Register
    ///
    /// X,Z,N = X-1
    ///
    /// Subtracts one from the X register setting the zero and negative flags as appropriate.
    DEX,

    /// Decrement Y Register
    ///
    /// Y,Z,N = Y-1
    ///
    /// Subtracts one from the Y register setting the zero and negative flags as appropriate.
    DEY,

    // Shifts
    /// Arithmetic Shift Left
    ///
    /// A,Z,C,N = M*2 or M,Z,C,N = M*2
    ///
    /// This operation shifts all the bits of the accumulator or memory contents one bit left. Bit 0
    /// is set to 0 and bit 7 is placed in the carry flag. The effect of this operation is to
    /// multiply the memory contents by 2 (ignoring 2's complement considerations), setting the
    /// carry if the result will not fit in 8 bits.
    ASL(ShiftAddressingMode),

    /// Logical Shift Right
    ///
    /// A,C,Z,N = A/2 or M,C,Z,N = M/2
    ///
    /// Each of the bits in A or M is shift one place to the right. The bit that was in bit 0 is
    /// shifted into the carry flag. Bit 7 is set to zero.
    LSR(ShiftAddressingMode),

    /// Rotate Left
    ///
    /// Move each of the bits in either A or M one place to the left. Bit 0 is filled with the
    /// current value of the carry flag whilst the old bit 7 becomes the new carry flag value.
    ROL(ShiftAddressingMode),

    /// Rotate Right
    ///
    /// Move each of the bits in either A or M one place to the right. Bit 7 is filled with the
    /// current value of the carry flag whilst the old bit 0 becomes the new carry flag value.
    ROR(ShiftAddressingMode),

    // Jumps & Calls
    /// Jump
    ///
    /// Sets the program counter to the address specified by the operand.
    JMP(JumpAddressingMode),

    /// Jump to Subroutine
    ///
    /// The JSR instruction pushes the address (minus one) of the return point on to the stack and
    /// then sets the program counter to the target memory address.
    JSR,

    /// Return from Subroutine
    ///
    /// The RTS instruction is used at the end of a subroutine to return to the calling routine. It
    /// pulls the program counter (minus one) from the stack.
    RTS,

    // Branches
    /// Branch if Carry Clear
    ///
    /// If the carry flag is clear then add the relative displacement to the program counter to
    /// cause a branch to a new location.
    BCC,

    /// Branch if Carry Set
    ///
    /// If the carry flag is set then add the relative displacement to the program counter to cause
    /// a branch to a new location.
    BCS,

    /// Branch if Equal
    ///
    /// If the zero flag is set then add the relative displacement to the program counter to cause a
    /// branch to a new location.
    BEQ,

    /// Branch if Minus
    ///
    /// If the negative flag is set then add the relative displacement to the program counter to
    /// cause a branch to a new location.
    BMI,

    /// Branch if Not Equal
    ///
    /// If the zero flag is clear then add the relative displacement to the program counter to cause
    /// a branch to a new location.
    BNE,

    /// Branch if Positive
    ///
    /// If the negative flag is clear then add the relative displacement to the program counter to
    /// cause a branch to a new location.
    BPL,

    /// Branch if Overflow Clear
    ///
    /// If the overflow flag is clear then add the relative displacement to the program counter to
    /// cause a branch to a new location.
    BVC,

    /// Branch if Overflow Set
    ///
    /// If the overflow flag is set then add the relative displacement to the program counter to
    /// cause a branch to a new location.
    BVS,

    // Status Flag Changes
    /// Clear Carry Flag
    ///
    /// C = 0
    ///
    /// Set the carry flag to zero.
    CLC,

    /// Clear Decimal Mode
    ///
    /// D = 0
    ///
    /// Sets the decimal mode flag to zero.
    CLD,

    /// Clear Interrupt Disable
    ///
    /// I = 0
    ///
    /// Clears the interrupt disable flag allowing normal interrupt requests to be serviced.
    CLI,

    /// Clear Overflow Flag
    ///
    /// V = 0
    ///
    /// Clears the overflow flag.
    CLV,

    /// Set Carry Flag
    ///
    /// C = 1
    ///
    /// Set the carry flag to one.
    SEC,

    /// Set Decimal Flag
    ///
    /// D = 1
    ///
    /// Set the decimal mode flag to one.
    SED,

    /// Set Interrupt Disable
    ///
    /// I = 1
    ///
    /// Set the interrupt disable flag to one.
    SEI,

    // System Functions
    /// Force Interrupt
    ///
    /// The BRK instruction forces the generation of an interrupt request. The program counter and
    /// processor status are pushed on the stack then the IRQ interrupt vector at $FFFE/F is loaded
    /// into the PC and the break flag in the status set to one.
    BRK,

    /// No Operation
    ///
    /// The NOP instruction causes no changes to the processor other than the normal incrementing of
    /// the program counter to the next instruction.
    NOP,

    /// Return from Interrupt
    ///
    /// The RTI instruction is used at the end of an interrupt processing routine. It pulls the
    /// processor flags from the stack followed by the program counter.
    RTI,
}

macro_rules! def_opcodes {
    ($($num:tt => $name:ident => $instr:ident$(($mode:path))*),* $(,)*) => {
        pub mod instructions {
            use super::*;

            $(
                pub const $name: Instruction = Instruction::$instr$(($mode))*;
            )*
        }

        impl Instruction {
            pub fn from_opcode(opcode: u8) -> Self {
                match opcode {
                    $(
                        $num => Instruction::$instr$(($mode))*,
                    )*
                    _ => panic!("Unrecognised opcode: {:#04x}", opcode)
                }
            }

            pub fn to_opcode(self) -> u8 {
                match self {
                    $(
                        Instruction::$instr $(($mode))* => $num,
                    )*
                }
            }
        }
    }
}

def_opcodes! {
    0x00 => BRK                  => BRK,
    0x01 => ORA_INDEXED_INDIRECT => ORA(FlexibleAddressingMode::IndexedIndirect),
    0x05 => ORA_ZERO_PAGE        => ORA(FlexibleAddressingMode::ZeroPage),
    0x06 => ASL_ZERO_PAGE        => ASL(ShiftAddressingMode::ZeroPage),
    0x08 => PHP                  => PHP,
    0x09 => ORA_IMMEDIATE        => ORA(FlexibleAddressingMode::Immediate),
    0x0A => ASL_ACCUMULATOR      => ASL(ShiftAddressingMode::Accumulator),
    0x0D => ORA_ABSOLUTE         => ORA(FlexibleAddressingMode::Absolute),
    0x0E => ASL_ABSOLUTE         => ASL(ShiftAddressingMode::Absolute),
    0x10 => BPL                  => BPL,
    0x11 => ORA_INDIRECT_INDEXED => ORA(FlexibleAddressingMode::IndirectIndexed),
    0x15 => ORA_ZERO_PAGE_X      => ORA(FlexibleAddressingMode::ZeroPageX),
    0x16 => ASL_ZERO_PAGE_X      => ASL(ShiftAddressingMode::ZeroPageX),
    0x18 => CLC                  => CLC,
    0x19 => ORA_ABSOLUTE_Y       => ORA(FlexibleAddressingMode::AbsoluteY),
    0x1D => ORA_ABSOLUTE_X       => ORA(FlexibleAddressingMode::AbsoluteX),
    0x1E => ASL_ABSOLUTE_X       => ASL(ShiftAddressingMode::AbsoluteX),
    0x20 => JSR                  => JSR,
    0x21 => AND_INDEXED_INDIRECT => AND(FlexibleAddressingMode::IndexedIndirect),
    0x24 => BIT_ZERO_PAGE        => BIT(BITAddressingMode::ZeroPage),
    0x25 => AND_ZERO_PAGE        => AND(FlexibleAddressingMode::ZeroPage),
    0x26 => ROL_ZERO_PAGE        => ROL(ShiftAddressingMode::ZeroPage),
    0x28 => PLP                  => PLP,
    0x29 => AND_IMMEDIATE        => AND(FlexibleAddressingMode::Immediate),
    0x2A => ROL_ACCUMULATOR      => ROL(ShiftAddressingMode::Accumulator),
    0x2C => BIT_ABSOLUTE         => BIT(BITAddressingMode::Absolute),
    0x2D => AND_ABSOLUTE         => AND(FlexibleAddressingMode::Absolute),
    0x2E => ROL_ABSOLUTE         => ROL(ShiftAddressingMode::Absolute),
    0x30 => BMI                  => BMI,
    0x31 => AND_INDIRECT_INDEXED => AND(FlexibleAddressingMode::IndirectIndexed),
    0x35 => AND_ZERO_PAGE_X      => AND(FlexibleAddressingMode::ZeroPageX),
    0x36 => ROL_ZERO_PAGE_X      => ROL(ShiftAddressingMode::ZeroPageX),
    0x38 => SEC                  => SEC,
    0x39 => AND_ABSOLUTE_Y       => AND(FlexibleAddressingMode::AbsoluteY),
    0x3D => AND_ABSOLUTE_X       => AND(FlexibleAddressingMode::AbsoluteX),
    0x3E => ROL_ABSOLUTE_X       => ROL(ShiftAddressingMode::AbsoluteX),
    0x40 => RTI                  => RTI,
    0x41 => EOR_INDEXED_INDIRECT => EOR(FlexibleAddressingMode::IndexedIndirect),
    0x45 => EOR_ZERO_PAGE        => EOR(FlexibleAddressingMode::ZeroPage),
    0x46 => LSR_ZERO_PAGE        => LSR(ShiftAddressingMode::ZeroPage),
    0x48 => PHA                  => PHA,
    0x49 => EOR_IMMEDIATE        => EOR(FlexibleAddressingMode::Immediate),
    0x4A => LSR_ACCUMULATOR      => LSR(ShiftAddressingMode::Accumulator),
    0x4C => JMP_ABSOLUTE         => JMP(JumpAddressingMode::Absolute),
    0x4D => EOR_ABSOLUTE         => EOR(FlexibleAddressingMode::Absolute),
    0x4E => LSR_ABSOLUTE         => LSR(ShiftAddressingMode::Absolute),
    0x50 => BVC                  => BVC,
    0x51 => EOR_INDIRECT_INDEXED => EOR(FlexibleAddressingMode::IndirectIndexed),
    0x55 => EOR_ZERO_PAGE_X      => EOR(FlexibleAddressingMode::ZeroPageX),
    0x56 => LSR_ZERO_PAGE_X      => LSR(ShiftAddressingMode::ZeroPageX),
    0x58 => CLI                  => CLI,
    0x59 => EOR_ABSOLUTE_Y       => EOR(FlexibleAddressingMode::AbsoluteY),
    0x5D => EOR_ABSOLUTE_X       => EOR(FlexibleAddressingMode::AbsoluteX),
    0x5E => LSR_ABSOLUTE_X       => LSR(ShiftAddressingMode::AbsoluteX),
    0x60 => RTS                  => RTS,
    0x61 => ADC_INDEXED_INDIRECT => ADC(FlexibleAddressingMode::IndexedIndirect),
    0x65 => ADC_ZERO_PAGE        => ADC(FlexibleAddressingMode::ZeroPage),
    0x66 => ROR_ZERO_PAGE        => ROR(ShiftAddressingMode::ZeroPage),
    0x68 => PLA                  => PLA,
    0x69 => ADC_IMMEDIATE        => ADC(FlexibleAddressingMode::Immediate),
    0x6A => ROR_ACCUMULATOR      => ROR(ShiftAddressingMode::Accumulator),
    0x6C => JMP_INDIRECT         => JMP(JumpAddressingMode::Indirect),
    0x6D => ADC_ABSOLUTE         => ADC(FlexibleAddressingMode::Absolute),
    0x6E => ROR_ABSOLUTE         => ROR(ShiftAddressingMode::Absolute),
    0x70 => BVS                  => BVS,
    0x71 => ADC_INDIRECT_INDEXED => ADC(FlexibleAddressingMode::IndirectIndexed),
    0x75 => ADC_ZERO_PAGE_X      => ADC(FlexibleAddressingMode::ZeroPageX),
    0x76 => ROR_ZERO_PAGE_X      => ROR(ShiftAddressingMode::ZeroPageX),
    0x78 => SEI                  => SEI,
    0x79 => ADC_ABSOLUTE_Y       => ADC(FlexibleAddressingMode::AbsoluteY),
    0x7D => ADC_ABSOLUTE_X       => ADC(FlexibleAddressingMode::AbsoluteX),
    0x7E => ROR_ABSOLUTE_X       => ROR(ShiftAddressingMode::AbsoluteX),
    0x81 => STA_INDEXED_INDIRECT => STA(StoreAddressingMode::IndexedIndirect),
    0x84 => STY_ZERO_PAGE        => STY(STYAddressingMode::ZeroPage),
    0x85 => STA_ZERO_PAGE        => STA(StoreAddressingMode::ZeroPage),
    0x86 => STX_ZERO_PAGE        => STX(STXAddressingMode::ZeroPage),
    0x88 => DEY                  => DEY,
    0x8A => TXA                  => TXA,
    0x8C => STY_ABSOLUTE         => STY(STYAddressingMode::Absolute),
    0x8D => STA_ABSOLUTE         => STA(StoreAddressingMode::Absolute),
    0x8E => STX_ABSOLUTE         => STX(STXAddressingMode::Absolute),
    0x90 => BCC                  => BCC,
    0x91 => STA_INDIRECT_INDEXED => STA(StoreAddressingMode::IndirectIndexed),
    0x94 => STY_ZERO_PAGE_X      => STY(STYAddressingMode::ZeroPageX),
    0x95 => STA_ZERO_PAGE_X      => STA(StoreAddressingMode::ZeroPageX),
    0x96 => STX_ZERO_PAGE_Y      => STX(STXAddressingMode::ZeroPageY),
    0x98 => TYA                  => TYA,
    0x99 => STA_ABSOLUTE_Y       => STA(StoreAddressingMode::AbsoluteY),
    0x9A => TXS                  => TXS,
    0x9D => STA_ABSOLUTE_X       => STA(StoreAddressingMode::AbsoluteX),
    0xA0 => LDY_IMMEDIATE        => LDY(LDYAddressingMode::Immediate),
    0xA1 => LDA_INDEXED_INDIRECT => LDA(FlexibleAddressingMode::IndexedIndirect),
    0xA2 => LDX_IMMEDIATE        => LDX(LDXAddressingMode::Immediate),
    0xA4 => LDY_ZERO_PAGE        => LDY(LDYAddressingMode::ZeroPage),
    0xA5 => LDA_ZERO_PAGE        => LDA(FlexibleAddressingMode::ZeroPage),
    0xA6 => LDX_ZERO_PAGE        => LDX(LDXAddressingMode::ZeroPage),
    0xA8 => TAY                  => TAY,
    0xA9 => LDA_IMMEDIATE        => LDA(FlexibleAddressingMode::Immediate),
    0xAA => TAX                  => TAX,
    0xAC => LDY_ABSOLUTE         => LDY(LDYAddressingMode::Absolute),
    0xAD => LDA_ABSOLUTE         => LDA(FlexibleAddressingMode::Absolute),
    0xAE => LDX_ABSOLUTE         => LDX(LDXAddressingMode::Absolute),
    0xB0 => BCS                  => BCS,
    0xB1 => LDA_INDIRECT_INDEXED => LDA(FlexibleAddressingMode::IndirectIndexed),
    0xB4 => LDY_ZERO_PAGE_X      => LDY(LDYAddressingMode::ZeroPageX),
    0xB5 => LDA_ZERO_PAGE_X      => LDA(FlexibleAddressingMode::ZeroPageX),
    0xB6 => LDX_ZERO_PAGE_Y      => LDX(LDXAddressingMode::ZeroPageY),
    0xB8 => CLV                  => CLV,
    0xB9 => LDA_ABSOLUTE_Y       => LDA(FlexibleAddressingMode::AbsoluteY),
    0xBA => TSX                  => TSX,
    0xBC => LDY_ABSOLUTE_X       => LDY(LDYAddressingMode::AbsoluteX),
    0xBD => LDA_ABSOLUTE_X       => LDA(FlexibleAddressingMode::AbsoluteX),
    0xBE => LDX_ABSOLUTE_Y       => LDX(LDXAddressingMode::AbsoluteY),
    0xC0 => CPY_IMMEDIATE        => CPY(CompareAddressingMode::Immediate),
    0xC1 => CMP_INDEXED_INDIRECT => CMP(FlexibleAddressingMode::IndexedIndirect),
    0xC4 => CPY_ZERO_PAGE        => CPY(CompareAddressingMode::ZeroPage),
    0xC5 => CMP_ZERO_PAGE        => CMP(FlexibleAddressingMode::ZeroPage),
    0xC6 => DEC_ZERO_PAGE        => DEC(IncDecAddressingMode::ZeroPage),
    0xC8 => INY                  => INY,
    0xC9 => CMP_IMMEDIATE        => CMP(FlexibleAddressingMode::Immediate),
    0xCA => DEX                  => DEX,
    0xCC => CPY_ABSOLUTE         => CPY(CompareAddressingMode::Absolute),
    0xCD => CMP_ABSOLUTE         => CMP(FlexibleAddressingMode::Absolute),
    0xCE => DEC_ABSOLUTE         => DEC(IncDecAddressingMode::Absolute),
    0xD0 => BNE                  => BNE,
    0xD1 => CMP_INDIRECT_INDEXED => CMP(FlexibleAddressingMode::IndirectIndexed),
    0xD5 => CMP_ZERO_PAGE_X      => CMP(FlexibleAddressingMode::ZeroPageX),
    0xD6 => DEC_ZERO_PAGE_X      => DEC(IncDecAddressingMode::ZeroPageX),
    0xD8 => CLD                  => CLD,
    0xD9 => CMP_ABSOLUTE_Y       => CMP(FlexibleAddressingMode::AbsoluteY),
    0xDD => CMP_ABSOLUTE_X       => CMP(FlexibleAddressingMode::AbsoluteX),
    0xDE => DEC_ABSOLUTE_X       => DEC(IncDecAddressingMode::AbsoluteX),
    0xE0 => CPX_IMMEDIATE        => CPX(CompareAddressingMode::Immediate),
    0xE1 => SBC_INDEXED_INDIRECT => SBC(FlexibleAddressingMode::IndexedIndirect),
    0xE4 => CPX_ZERO_PAGE        => CPX(CompareAddressingMode::ZeroPage),
    0xE5 => SBC_ZERO_PAGE        => SBC(FlexibleAddressingMode::ZeroPage),
    0xE6 => INC_ZERO_PAGE        => INC(IncDecAddressingMode::ZeroPage),
    0xE8 => INX                  => INX,
    0xE9 => SBC_IMMEDIATE        => SBC(FlexibleAddressingMode::Immediate),
    0xEA => NOP                  => NOP,
    0xEC => CPX_ABSOLUTE         => CPX(CompareAddressingMode::Absolute),
    0xED => SBC_ABSOLUTE         => SBC(FlexibleAddressingMode::Absolute),
    0xEE => INC_ABSOLUTE         => INC(IncDecAddressingMode::Absolute),
    0xF0 => BEQ                  => BEQ,
    0xF1 => SBC_INDIRECT_INDEXED => SBC(FlexibleAddressingMode::IndirectIndexed),
    0xF5 => SBC_ZERO_PAGE_X      => SBC(FlexibleAddressingMode::ZeroPageX),
    0xF6 => INC_ZERO_PAGE_X      => INC(IncDecAddressingMode::ZeroPageX),
    0xF8 => SED                  => SED,
    0xF9 => SBC_ABSOLUTE_Y       => SBC(FlexibleAddressingMode::AbsoluteY),
    0xFD => SBC_ABSOLUTE_X       => SBC(FlexibleAddressingMode::AbsoluteX),
    0xFE => INC_ABSOLUTE_X       => INC(IncDecAddressingMode::AbsoluteX),
}
