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
    ($($num:tt => $name:ident => $instr:ident$(($mode_cat:ident::$mode:ident))*),* $(,)*) => {
        pub mod instructions {
            use super::*;

            $(
                pub const $name: Instruction = Instruction::$instr$(($mode_cat::$mode))*;
            )*
        }

        impl Instruction {
            pub fn from_opcode(opcode: u8) -> Self {
                use self::Instruction::*;

                match opcode {
                    $(
                        $num => $instr$(($mode_cat::$mode))*,
                    )*
                    _ => panic!("Unrecognised opcode: {:#04x}", opcode)
                }
            }

            pub fn to_opcode(self) -> u8 {
                match self {
                    $(
                        Instruction::$instr $(($mode_cat::$mode))* => $num,
                    )*
                }
            }
        }
    }
}

def_opcodes! {
    0x00 => BRK => BRK,
    0x01 => ORAIndexedIndirect => ORA(FlexibleAddressingMode::IndexedIndirect),
    0x05 => ORAZeroPage => ORA(FlexibleAddressingMode::ZeroPage),
    0x06 => ASLZeroPage => ASL(ShiftAddressingMode::ZeroPage),
    0x08 => PHP => PHP,
    0x09 => ORAImmediate => ORA(FlexibleAddressingMode::Immediate),
    0x0A => ASLAccumulator => ASL(ShiftAddressingMode::Accumulator),
    0x0D => ORAAbsolute => ORA(FlexibleAddressingMode::Absolute),
    0x0E => ASLAbsolute => ASL(ShiftAddressingMode::Absolute),
    0x10 => BPL => BPL,
    0x11 => ORAIndirectIndexed => ORA(FlexibleAddressingMode::IndirectIndexed),
    0x15 => ORAZeroPageX => ORA(FlexibleAddressingMode::ZeroPageX),
    0x16 => ASLZeroPageX => ASL(ShiftAddressingMode::ZeroPageX),
    0x18 => CLC => CLC,
    0x19 => ORAAbsoluteY => ORA(FlexibleAddressingMode::AbsoluteY),
    0x1D => ORAAbsoluteX => ORA(FlexibleAddressingMode::AbsoluteX),
    0x1E => ASLAbsoluteX => ASL(ShiftAddressingMode::AbsoluteX),
    0x20 => JSR => JSR,
    0x21 => ANDIndexedIndirect => AND(FlexibleAddressingMode::IndexedIndirect),
    0x24 => BITZeroPage => BIT(BITAddressingMode::ZeroPage),
    0x25 => ANDZeroPage => AND(FlexibleAddressingMode::ZeroPage),
    0x26 => ROLZeroPage => ROL(ShiftAddressingMode::ZeroPage),
    0x28 => PLP => PLP,
    0x29 => ANDImmediate => AND(FlexibleAddressingMode::Immediate),
    0x2A => ROLAccumulator => ROL(ShiftAddressingMode::Accumulator),
    0x2C => BITAbsolute => BIT(BITAddressingMode::Absolute),
    0x2D => ANDAbsolute => AND(FlexibleAddressingMode::Absolute),
    0x2E => ROLAbsolute => ROL(ShiftAddressingMode::Absolute),
    0x30 => BMI => BMI,
    0x31 => ANDIndirectIndexed => AND(FlexibleAddressingMode::IndirectIndexed),
    0x35 => ANDZeroPageX => AND(FlexibleAddressingMode::ZeroPageX),
    0x36 => ROLZeroPageX => ROL(ShiftAddressingMode::ZeroPageX),
    0x38 => SEC => SEC,
    0x39 => ANDAbsoluteY => AND(FlexibleAddressingMode::AbsoluteY),
    0x3D => ANDAbsoluteX => AND(FlexibleAddressingMode::AbsoluteX),
    0x3E => ROLAbsoluteX => ROL(ShiftAddressingMode::AbsoluteX),
    0x40 => RTI => RTI,
    0x41 => EORIndexedIndirect => EOR(FlexibleAddressingMode::IndexedIndirect),
    0x45 => EORZeroPage => EOR(FlexibleAddressingMode::ZeroPage),
    0x46 => LSRZeroPage => LSR(ShiftAddressingMode::ZeroPage),
    0x48 => PHA => PHA,
    0x49 => EORImmediate => EOR(FlexibleAddressingMode::Immediate),
    0x4A => LSRAccumulator => LSR(ShiftAddressingMode::Accumulator),
    0x4C => JMPAbsolute => JMP(JumpAddressingMode::Absolute),
    0x4D => EORAbsolute => EOR(FlexibleAddressingMode::Absolute),
    0x4E => LSRAbsolute => LSR(ShiftAddressingMode::Absolute),
    0x50 => BVC => BVC,
    0x51 => EORIndirectIndexed => EOR(FlexibleAddressingMode::IndirectIndexed),
    0x55 => EORZeroPageX => EOR(FlexibleAddressingMode::ZeroPageX),
    0x56 => LSRZeroPageX => LSR(ShiftAddressingMode::ZeroPageX),
    0x58 => CLI => CLI,
    0x59 => EORAbsoluteY => EOR(FlexibleAddressingMode::AbsoluteY),
    0x5D => EORAbsoluteX => EOR(FlexibleAddressingMode::AbsoluteX),
    0x5E => LSRAbsoluteX => LSR(ShiftAddressingMode::AbsoluteX),
    0x60 => RTS => RTS,
    0x61 => ADCIndexedIndirect => ADC(FlexibleAddressingMode::IndexedIndirect),
    0x65 => ADCZeroPage => ADC(FlexibleAddressingMode::ZeroPage),
    0x66 => RORZeroPage => ROR(ShiftAddressingMode::ZeroPage),
    0x68 => PLA => PLA,
    0x69 => ADCImmediate => ADC(FlexibleAddressingMode::Immediate),
    0x6A => RORAccumulator => ROR(ShiftAddressingMode::Accumulator),
    0x6C => JMPIndirect => JMP(JumpAddressingMode::Indirect),
    0x6D => ADCAbsolute => ADC(FlexibleAddressingMode::Absolute),
    0x6E => RORAbsolute => ROR(ShiftAddressingMode::Absolute),
    0x70 => BVS => BVS,
    0x71 => ADCIndirectIndexed => ADC(FlexibleAddressingMode::IndirectIndexed),
    0x75 => ADCZeroPageX => ADC(FlexibleAddressingMode::ZeroPageX),
    0x76 => RORZeroPageX => ROR(ShiftAddressingMode::ZeroPageX),
    0x78 => SEI => SEI,
    0x79 => ADCAbsoluteY => ADC(FlexibleAddressingMode::AbsoluteY),
    0x7D => ADCAbsoluteX => ADC(FlexibleAddressingMode::AbsoluteX),
    0x7E => RORAbsoluteX => ROR(ShiftAddressingMode::AbsoluteX),
    0x81 => STAIndexedIndirect => STA(StoreAddressingMode::IndexedIndirect),
    0x84 => STYZeroPage => STY(STYAddressingMode::ZeroPage),
    0x85 => STAZeroPage => STA(StoreAddressingMode::ZeroPage),
    0x86 => STXZeroPage => STX(STXAddressingMode::ZeroPage),
    0x88 => DEY => DEY,
    0x8A => TXA => TXA,
    0x8C => STYAbsolute => STY(STYAddressingMode::Absolute),
    0x8D => STAAbsolute => STA(StoreAddressingMode::Absolute),
    0x8E => STXAbsolute => STX(STXAddressingMode::Absolute),
    0x90 => BCC => BCC,
    0x91 => STAIndirectIndexed => STA(StoreAddressingMode::IndirectIndexed),
    0x94 => STYZeroPageX => STY(STYAddressingMode::ZeroPageX),
    0x95 => STAZeroPageX => STA(StoreAddressingMode::ZeroPageX),
    0x96 => STXZeroPageY => STX(STXAddressingMode::ZeroPageY),
    0x98 => TYA => TYA,
    0x99 => STAAbsoluteY => STA(StoreAddressingMode::AbsoluteY),
    0x9A => TXS => TXS,
    0x9D => STAAbsoluteX => STA(StoreAddressingMode::AbsoluteX),
    0xA0 => LDYImmediate => LDY(LDYAddressingMode::Immediate),
    0xA1 => LDAIndexedIndirect => LDA(FlexibleAddressingMode::IndexedIndirect),
    0xA2 => LDXImmediate => LDX(LDXAddressingMode::Immediate),
    0xA4 => LDYZeroPage => LDY(LDYAddressingMode::ZeroPage),
    0xA5 => LDAZeroPage => LDA(FlexibleAddressingMode::ZeroPage),
    0xA6 => LDXZeroPage => LDX(LDXAddressingMode::ZeroPage),
    0xA8 => TAY => TAY,
    0xA9 => LDAImmediate => LDA(FlexibleAddressingMode::Immediate),
    0xAA => TAX => TAX,
    0xAC => LDYAbsolute => LDY(LDYAddressingMode::Absolute),
    0xAD => LDAAbsolute => LDA(FlexibleAddressingMode::Absolute),
    0xAE => LDXAbsolute => LDX(LDXAddressingMode::Absolute),
    0xB0 => BCS => BCS,
    0xB1 => LDAIndirectIndexed => LDA(FlexibleAddressingMode::IndirectIndexed),
    0xB4 => LDYZeroPageX => LDY(LDYAddressingMode::ZeroPageX),
    0xB5 => LDAZeroPageX => LDA(FlexibleAddressingMode::ZeroPageX),
    0xB6 => LDXZeroPageY => LDX(LDXAddressingMode::ZeroPageY),
    0xB8 => CLV => CLV,
    0xB9 => LDAAbsoluteY => LDA(FlexibleAddressingMode::AbsoluteY),
    0xBA => TSX => TSX,
    0xBC => LDYAbsoluteX => LDY(LDYAddressingMode::AbsoluteX),
    0xBD => LDAAbsoluteX => LDA(FlexibleAddressingMode::AbsoluteX),
    0xBE => LDXAbsoluteY => LDX(LDXAddressingMode::AbsoluteY),
    0xC0 => CPYImmediate => CPY(CompareAddressingMode::Immediate),
    0xC1 => CMPIndexedIndirect => CMP(FlexibleAddressingMode::IndexedIndirect),
    0xC4 => CPYZeroPage => CPY(CompareAddressingMode::ZeroPage),
    0xC5 => CMPZeroPage => CMP(FlexibleAddressingMode::ZeroPage),
    0xC6 => DECZeroPage => DEC(IncDecAddressingMode::ZeroPage),
    0xC8 => INY => INY,
    0xC9 => CMPImmediate => CMP(FlexibleAddressingMode::Immediate),
    0xCA => DEX => DEX,
    0xCC => CPYAbsolute => CPY(CompareAddressingMode::Absolute),
    0xCD => CMPAbsolute => CMP(FlexibleAddressingMode::Absolute),
    0xCE => DECAbsolute => DEC(IncDecAddressingMode::Absolute),
    0xD0 => BNE => BNE,
    0xD1 => CMPIndirectIndexed => CMP(FlexibleAddressingMode::IndirectIndexed),
    0xD5 => CMPZeroPageX => CMP(FlexibleAddressingMode::ZeroPageX),
    0xD6 => DECZeroPageX => DEC(IncDecAddressingMode::ZeroPageX),
    0xD8 => CLD => CLD,
    0xD9 => CMPAbsoluteY => CMP(FlexibleAddressingMode::AbsoluteY),
    0xDD => CMPAbsoluteX => CMP(FlexibleAddressingMode::AbsoluteX),
    0xDE => DECAbsoluteX => DEC(IncDecAddressingMode::AbsoluteX),
    0xE0 => CPXImmediate => CPX(CompareAddressingMode::Immediate),
    0xE1 => SBCIndexedIndirect => SBC(FlexibleAddressingMode::IndexedIndirect),
    0xE4 => CPXZeroPage => CPX(CompareAddressingMode::ZeroPage),
    0xE5 => SBCZeroPage => SBC(FlexibleAddressingMode::ZeroPage),
    0xE6 => INCZeroPage => INC(IncDecAddressingMode::ZeroPage),
    0xE8 => INX => INX,
    0xE9 => SBCImmediate => SBC(FlexibleAddressingMode::Immediate),
    0xEA => NOP => NOP,
    0xEC => CPXAbsolute => CPX(CompareAddressingMode::Absolute),
    0xED => SBCAbsolute => SBC(FlexibleAddressingMode::Absolute),
    0xEE => INCAbsolute => INC(IncDecAddressingMode::Absolute),
    0xF0 => BEQ => BEQ,
    0xF1 => SBCIndirectIndexed => SBC(FlexibleAddressingMode::IndirectIndexed),
    0xF5 => SBCZeroPageX => SBC(FlexibleAddressingMode::ZeroPageX),
    0xF6 => INCZeroPageX => INC(IncDecAddressingMode::ZeroPageX),
    0xF8 => SED => SED,
    0xF9 => SBCAbsoluteY => SBC(FlexibleAddressingMode::AbsoluteY),
    0xFD => SBCAbsoluteX => SBC(FlexibleAddressingMode::AbsoluteX),
    0xFE => INCAbsoluteX => INC(IncDecAddressingMode::AbsoluteX),
}
