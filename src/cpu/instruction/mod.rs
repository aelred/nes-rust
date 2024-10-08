use super::addressing_modes::BITAddressingMode;
use super::addressing_modes::CompareAddressingMode;
use super::addressing_modes::FlexibleAddressingMode;
use super::addressing_modes::IncDecAddressingMode;
use super::addressing_modes::JumpAddressingMode;
use super::addressing_modes::LAXAddressingMode;
use super::addressing_modes::LDXAddressingMode;
use super::addressing_modes::LDYAddressingMode;
use super::addressing_modes::SAXAddressingMode;
use super::addressing_modes::STXAddressingMode;
use super::addressing_modes::STYAddressingMode;
use super::addressing_modes::ShiftAddressingMode;
use super::addressing_modes::StoreAddressingMode;

pub mod arithmetic;
pub mod branch;
pub mod inc_dec;
pub mod jump;
pub mod load_store;
pub mod logical;
pub mod shift;
pub mod stack;
pub mod status;
pub mod system;
pub mod transfer;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

    // Unofficial Opcodes
    /// Ignore
    ///
    /// Reads from memory at the specified address and ignores the value. Affects no register nor
    /// flags.
    IGN(IncDecAddressingMode),

    /// Skip Byte
    ///
    /// These unofficial opcodes just read an immediate byte and skip it, like a different address
    /// mode of NOP.
    SKB,

    /// Load Accumulator And X Register
    ///
    /// Shortcut for LDA value then TAX.
    LAX(LAXAddressingMode),

    /// Store Accumulator And X Register
    ///
    /// Stores the bitwise AND of A and X. As with STA and STX, no flags are affected.
    SAX(SAXAddressingMode),

    /// Decrement Memory And Compare
    ///
    /// Equivalent to DEC value then CMP value, except supporting more addressing modes.
    DCP(StoreAddressingMode),

    /// Increment Memory And Subtract With Carry
    ///
    /// Equivalent to INC value then SBC value, except supporting more addressing modes.
    ISC(StoreAddressingMode),

    /// Arithmetic Shift Left And Logical Inclusive OR
    ///
    /// Equivalent to ASL value then ORA value, except supporting more addressing modes.
    SLO(StoreAddressingMode),

    /// Rotate Left And Logical AND
    ///
    /// Equivalent to ROL value then AND value, except supporting more addressing modes.
    RLA(StoreAddressingMode),

    /// Logical Shift Right And Exclusive OR
    ///
    /// Equivalent to LSR value then EOR value, except supporting more addressing modes.
    SRE(StoreAddressingMode),

    /// Rotate Right And Add With Carry
    ///
    /// Equivalent to ROR value then ADC value, except supporting more addressing modes.
    RRA(StoreAddressingMode),
}

macro_rules! def_opcodes {
    ($($num:tt => $name:ident $(=> $instr:ident$(($mode:path))*)*),* $(,)*) => {
        pub mod instructions {
            use super::*;

            $(
                $(
                    pub const $name: Instruction = Instruction::$instr$(($mode))*;
                )*
            )*
        }

        impl Instruction {
            pub fn from_opcode(opcode: u8) -> Self {
                use super::instructions::*;

                match opcode {
                    $(
                        $num => $name,
                    )*
                    _ => panic!("Unrecognised opcode: {:#04x}", opcode)
                }
            }

            pub fn to_opcode(self) -> u8 {
                use super::instructions::*;

                // Some instructions have multiple opcodes associated.
                // We only return the first (lowest) one.
                #[allow(unreachable_patterns)]
                match self {
                    $(
                        $name => $num,
                    )*
                }
            }
        }
    }
}

def_opcodes! {
    0x00 => BRK     => BRK,
    0x01 => ORA_IDX => ORA(FlexibleAddressingMode::IndexedIndirect),
    0x03 => SLO_IDX => SLO(StoreAddressingMode::IndexedIndirect),
    0x04 => IGN_ZPA => IGN(IncDecAddressingMode::ZeroPage),
    0x05 => ORA_ZPA => ORA(FlexibleAddressingMode::ZeroPage),
    0x06 => ASL_ZPA => ASL(ShiftAddressingMode::ZeroPage),
    0x07 => SLO_ZPA => SLO(StoreAddressingMode::ZeroPage),
    0x08 => PHP     => PHP,
    0x09 => ORA_IMM => ORA(FlexibleAddressingMode::Immediate),
    0x0A => ASL_ACC => ASL(ShiftAddressingMode::Accumulator),
    0x0C => IGN_ABS => IGN(IncDecAddressingMode::Absolute),
    0x0D => ORA_ABS => ORA(FlexibleAddressingMode::Absolute),
    0x0E => ASL_ABS => ASL(ShiftAddressingMode::Absolute),
    0x0F => SLO_ABS => SLO(StoreAddressingMode::Absolute),
    0x10 => BPL     => BPL,
    0x11 => ORA_IDY => ORA(FlexibleAddressingMode::IndirectIndexed),
    0x13 => SOL_IDY => SLO(StoreAddressingMode::IndirectIndexed),
    0x15 => ORA_ZPX => ORA(FlexibleAddressingMode::ZeroPageX),
    0x14 => IGN_ZPX => IGN(IncDecAddressingMode::ZeroPageX),
    0x16 => ASL_ZPX => ASL(ShiftAddressingMode::ZeroPageX),
    0x17 => SLO_ZPX => SLO(StoreAddressingMode::ZeroPageX),
    0x18 => CLC     => CLC,
    0x19 => ORA_ABY => ORA(FlexibleAddressingMode::AbsoluteY),
    0x1A => NOP,
    0x1B => SLO_ABY => SLO(StoreAddressingMode::AbsoluteY),
    0x1C => IGN_ABX => IGN(IncDecAddressingMode::AbsoluteX),
    0x1D => ORA_ABX => ORA(FlexibleAddressingMode::AbsoluteX),
    0x1E => ASL_ABX => ASL(ShiftAddressingMode::AbsoluteX),
    0x1F => SLO_ABX => SLO(StoreAddressingMode::AbsoluteX),
    0x20 => JSR     => JSR,
    0x21 => AND_IDX => AND(FlexibleAddressingMode::IndexedIndirect),
    0x23 => RLA_IDX => RLA(StoreAddressingMode::IndexedIndirect),
    0x24 => BIT_ZPA => BIT(BITAddressingMode::ZeroPage),
    0x25 => AND_ZPA => AND(FlexibleAddressingMode::ZeroPage),
    0x26 => ROL_ZPA => ROL(ShiftAddressingMode::ZeroPage),
    0x27 => RLA_ZPA => RLA(StoreAddressingMode::ZeroPage),
    0x28 => PLP     => PLP,
    0x29 => AND_IMM => AND(FlexibleAddressingMode::Immediate),
    0x2A => ROL_ACC => ROL(ShiftAddressingMode::Accumulator),
    0x2C => BIT_ABS => BIT(BITAddressingMode::Absolute),
    0x2D => AND_ABS => AND(FlexibleAddressingMode::Absolute),
    0x2E => ROL_ABS => ROL(ShiftAddressingMode::Absolute),
    0x2F => RLA_ABS => RLA(StoreAddressingMode::Absolute),
    0x30 => BMI     => BMI,
    0x31 => AND_IDY => AND(FlexibleAddressingMode::IndirectIndexed),
    0x33 => RLA_IDY => RLA(StoreAddressingMode::IndirectIndexed),
    0x34 => IGN_ZPX,
    0x35 => AND_ZPX => AND(FlexibleAddressingMode::ZeroPageX),
    0x36 => ROL_ZPX => ROL(ShiftAddressingMode::ZeroPageX),
    0x37 => RLA_ZPX => RLA(StoreAddressingMode::ZeroPageX),
    0x38 => SEC     => SEC,
    0x39 => AND_ABY => AND(FlexibleAddressingMode::AbsoluteY),
    0x3A => NOP,
    0x3B => RLA_ABY => RLA(StoreAddressingMode::AbsoluteY),
    0x3C => IGN_ABX,
    0x3D => AND_ABX => AND(FlexibleAddressingMode::AbsoluteX),
    0x3E => ROL_ABX => ROL(ShiftAddressingMode::AbsoluteX),
    0x3F => RLA_ABX => RLA(StoreAddressingMode::AbsoluteX),
    0x40 => RTI     => RTI,
    0x41 => EOR_IDX => EOR(FlexibleAddressingMode::IndexedIndirect),
    0x43 => SRE_IDX => SRE(StoreAddressingMode::IndexedIndirect),
    0x44 => IGN_ZPA,
    0x45 => EOR_ZPA => EOR(FlexibleAddressingMode::ZeroPage),
    0x46 => LSR_ZPA => LSR(ShiftAddressingMode::ZeroPage),
    0x47 => SRE_ZPA => SRE(StoreAddressingMode::ZeroPage),
    0x48 => PHA     => PHA,
    0x49 => EOR_IMM => EOR(FlexibleAddressingMode::Immediate),
    0x4A => LSR_ACC => LSR(ShiftAddressingMode::Accumulator),
    0x4C => JMP_ABS => JMP(JumpAddressingMode::Absolute),
    0x4D => EOR_ABS => EOR(FlexibleAddressingMode::Absolute),
    0x4E => LSR_ABS => LSR(ShiftAddressingMode::Absolute),
    0x4F => SRE_ABS => SRE(StoreAddressingMode::Absolute),
    0x50 => BVC     => BVC,
    0x51 => EOR_IDY => EOR(FlexibleAddressingMode::IndirectIndexed),
    0x53 => SRE_IDY => SRE(StoreAddressingMode::IndirectIndexed),
    0x54 => IGN_ZPX,
    0x55 => EOR_ZPX => EOR(FlexibleAddressingMode::ZeroPageX),
    0x56 => LSR_ZPX => LSR(ShiftAddressingMode::ZeroPageX),
    0x57 => SRE_ZPX => SRE(StoreAddressingMode::ZeroPageX),
    0x58 => CLI     => CLI,
    0x59 => EOR_ABY => EOR(FlexibleAddressingMode::AbsoluteY),
    0x5A => NOP,
    0x5B => SRE_ABY => SRE(StoreAddressingMode::AbsoluteY),
    0x5C => IGN_ABX,
    0x5D => EOR_ABX => EOR(FlexibleAddressingMode::AbsoluteX),
    0x5E => LSR_ABX => LSR(ShiftAddressingMode::AbsoluteX),
    0x5F => SRE_ABX => SRE(StoreAddressingMode::AbsoluteX),
    0x60 => RTS     => RTS,
    0x61 => ADC_IDX => ADC(FlexibleAddressingMode::IndexedIndirect),
    0x63 => RRA_IDX => RRA(StoreAddressingMode::IndexedIndirect),
    0x64 => IGN_ZPA,
    0x65 => ADC_ZPA => ADC(FlexibleAddressingMode::ZeroPage),
    0x66 => ROR_ZPA => ROR(ShiftAddressingMode::ZeroPage),
    0x67 => RRA_ZPA => RRA(StoreAddressingMode::ZeroPage),
    0x68 => PLA     => PLA,
    0x69 => ADC_IMM => ADC(FlexibleAddressingMode::Immediate),
    0x6A => ROR_ACC => ROR(ShiftAddressingMode::Accumulator),
    0x6C => JMP_IND => JMP(JumpAddressingMode::Indirect),
    0x6D => ADC_ABS => ADC(FlexibleAddressingMode::Absolute),
    0x6E => ROR_ABS => ROR(ShiftAddressingMode::Absolute),
    0x6F => RRA_ABS => RRA(StoreAddressingMode::Absolute),
    0x70 => BVS     => BVS,
    0x71 => ADC_IDY => ADC(FlexibleAddressingMode::IndirectIndexed),
    0x73 => RRA_IDY => RRA(StoreAddressingMode::IndirectIndexed),
    0x74 => IGN_ZPX,
    0x75 => ADC_ZPX => ADC(FlexibleAddressingMode::ZeroPageX),
    0x76 => ROR_ZPX => ROR(ShiftAddressingMode::ZeroPageX),
    0x77 => RRA_ZPX => RRA(StoreAddressingMode::ZeroPageX),
    0x78 => SEI     => SEI,
    0x79 => ADC_ABY => ADC(FlexibleAddressingMode::AbsoluteY),
    0x7A => NOP,
    0x7B => RRA_ABY => RRA(StoreAddressingMode::AbsoluteY),
    0x7C => IGN_ABX,
    0x7D => ADC_ABX => ADC(FlexibleAddressingMode::AbsoluteX),
    0x7E => ROR_ABX => ROR(ShiftAddressingMode::AbsoluteX),
    0x7F => RRA_ABX => RRA(StoreAddressingMode::AbsoluteX),
    0x80 => SKB     => SKB,
    0x81 => STA_IDX => STA(StoreAddressingMode::IndexedIndirect),
    0x82 => SKB,
    0x83 => SAX_IDX => SAX(SAXAddressingMode::IndexedIndirect),
    0x84 => STY_ZPA => STY(STYAddressingMode::ZeroPage),
    0x85 => STA_ZPA => STA(StoreAddressingMode::ZeroPage),
    0x86 => STX_ZPA => STX(STXAddressingMode::ZeroPage),
    0x87 => SAX_ZPA => SAX(SAXAddressingMode::ZeroPage),
    0x88 => DEY     => DEY,
    0x89 => SKB,
    0x8A => TXA     => TXA,
    0x8C => STY_ABS => STY(STYAddressingMode::Absolute),
    0x8D => STA_ABS => STA(StoreAddressingMode::Absolute),
    0x8E => STX_ABS => STX(STXAddressingMode::Absolute),
    0x8F => SAX_ABS => SAX(SAXAddressingMode::Absolute),
    0x90 => BCC     => BCC,
    0x91 => STA_IDY => STA(StoreAddressingMode::IndirectIndexed),
    0x94 => STY_ZPX => STY(STYAddressingMode::ZeroPageX),
    0x95 => STA_ZPX => STA(StoreAddressingMode::ZeroPageX),
    0x96 => STX_ZPY => STX(STXAddressingMode::ZeroPageY),
    0x97 => SAX_ZPY => SAX(SAXAddressingMode::ZeroPageY),
    0x98 => TYA     => TYA,
    0x99 => STA_ABY => STA(StoreAddressingMode::AbsoluteY),
    0x9A => TXS     => TXS,
    0x9D => STA_ABX => STA(StoreAddressingMode::AbsoluteX),
    0xA0 => LDY_IMM => LDY(LDYAddressingMode::Immediate),
    0xA1 => LDA_IDX => LDA(FlexibleAddressingMode::IndexedIndirect),
    0xA2 => LDX_IMM => LDX(LDXAddressingMode::Immediate),
    0xA3 => LAX_IDX => LAX(LAXAddressingMode::IndexedIndirect),
    0xA4 => LDY_ZPA => LDY(LDYAddressingMode::ZeroPage),
    0xA5 => LDA_ZPA => LDA(FlexibleAddressingMode::ZeroPage),
    0xA6 => LDX_ZPA => LDX(LDXAddressingMode::ZeroPage),
    0xA7 => LAX_ZPA => LAX(LAXAddressingMode::ZeroPage),
    0xA8 => TAY     => TAY,
    0xA9 => LDA_IMM => LDA(FlexibleAddressingMode::Immediate),
    0xAA => TAX     => TAX,
    0xAC => LDY_ABS => LDY(LDYAddressingMode::Absolute),
    0xAD => LDA_ABS => LDA(FlexibleAddressingMode::Absolute),
    0xAE => LDX_ABS => LDX(LDXAddressingMode::Absolute),
    0xAF => LAX_ABS => LAX(LAXAddressingMode::Absolute),
    0xB0 => BCS     => BCS,
    0xB1 => LDA_IDY => LDA(FlexibleAddressingMode::IndirectIndexed),
    0xB3 => LAX_IDY => LAX(LAXAddressingMode::IndirectIndexed),
    0xB4 => LDY_ZPX => LDY(LDYAddressingMode::ZeroPageX),
    0xB5 => LDA_ZPX => LDA(FlexibleAddressingMode::ZeroPageX),
    0xB6 => LDX_ZPY => LDX(LDXAddressingMode::ZeroPageY),
    0xB7 => LAX_ZPY => LAX(LAXAddressingMode::ZeroPageY),
    0xB8 => CLV     => CLV,
    0xB9 => LDA_ABY => LDA(FlexibleAddressingMode::AbsoluteY),
    0xBA => TSX     => TSX,
    0xBC => LDY_ABX => LDY(LDYAddressingMode::AbsoluteX),
    0xBD => LDA_ABX => LDA(FlexibleAddressingMode::AbsoluteX),
    0xBE => LDX_ABY => LDX(LDXAddressingMode::AbsoluteY),
    0xBF => LAX_ABY => LAX(LAXAddressingMode::AbsoluteY),
    0xC0 => CPY_IMM => CPY(CompareAddressingMode::Immediate),
    0xC1 => CMP_IDX => CMP(FlexibleAddressingMode::IndexedIndirect),
    0xC2 => SKB,
    0xC3 => DCP_IDX => DCP(StoreAddressingMode::IndexedIndirect),
    0xC4 => CPY_ZPA => CPY(CompareAddressingMode::ZeroPage),
    0xC5 => CMP_ZPA => CMP(FlexibleAddressingMode::ZeroPage),
    0xC6 => DEC_ZPA => DEC(IncDecAddressingMode::ZeroPage),
    0xC7 => DCP_ZPA => DCP(StoreAddressingMode::ZeroPage),
    0xC8 => INY     => INY,
    0xC9 => CMP_IMM => CMP(FlexibleAddressingMode::Immediate),
    0xCA => DEX     => DEX,
    0xCC => CPY_ABS => CPY(CompareAddressingMode::Absolute),
    0xCD => CMP_ABS => CMP(FlexibleAddressingMode::Absolute),
    0xCE => DEC_ABS => DEC(IncDecAddressingMode::Absolute),
    0xCF => DCP_ABS => DCP(StoreAddressingMode::Absolute),
    0xD0 => BNE     => BNE,
    0xD1 => CMP_IDY => CMP(FlexibleAddressingMode::IndirectIndexed),
    0xD3 => DCP_IDY => DCP(StoreAddressingMode::IndirectIndexed),
    0xD4 => IGN_ZPX,
    0xD5 => CMP_ZPX => CMP(FlexibleAddressingMode::ZeroPageX),
    0xD6 => DEC_ZPX => DEC(IncDecAddressingMode::ZeroPageX),
    0xD7 => DCP_ZPX => DCP(StoreAddressingMode::ZeroPageX),
    0xD8 => CLD     => CLD,
    0xD9 => CMP_ABY => CMP(FlexibleAddressingMode::AbsoluteY),
    0xDA => NOP,
    0xDB => DCP_ABY => DCP(StoreAddressingMode::AbsoluteY),
    0xDC => IGN_ABX,
    0xDD => CMP_ABX => CMP(FlexibleAddressingMode::AbsoluteX),
    0xDE => DEC_ABX => DEC(IncDecAddressingMode::AbsoluteX),
    0xDF => DCP_ABX => DCP(StoreAddressingMode::AbsoluteX),
    0xE0 => CPX_IMM => CPX(CompareAddressingMode::Immediate),
    0xE1 => SBC_IDX => SBC(FlexibleAddressingMode::IndexedIndirect),
    0xE2 => SKB,
    0xE3 => ISC_IDX => ISC(StoreAddressingMode::IndexedIndirect),
    0xE4 => CPX_ZPA => CPX(CompareAddressingMode::ZeroPage),
    0xE5 => SBC_ZPA => SBC(FlexibleAddressingMode::ZeroPage),
    0xE6 => INC_ZPA => INC(IncDecAddressingMode::ZeroPage),
    0xE7 => ISC_ZPA => ISC(StoreAddressingMode::ZeroPage),
    0xE8 => INX     => INX,
    0xE9 => SBC_IMM => SBC(FlexibleAddressingMode::Immediate),
    0xEA => NOP     => NOP,
    0xEB => ADC_IMM,
    0xEC => CPX_ABS => CPX(CompareAddressingMode::Absolute),
    0xED => SBC_ABS => SBC(FlexibleAddressingMode::Absolute),
    0xEE => INC_ABS => INC(IncDecAddressingMode::Absolute),
    0xEF => ISC_ABS => ISC(StoreAddressingMode::Absolute),
    0xF0 => BEQ     => BEQ,
    0xF1 => SBC_IDY => SBC(FlexibleAddressingMode::IndirectIndexed),
    0xF3 => ISC_IDY => ISC(StoreAddressingMode::IndirectIndexed),
    0xF4 => IGN_ZPX,
    0xF5 => SBC_ZPX => SBC(FlexibleAddressingMode::ZeroPageX),
    0xF6 => INC_ZPX => INC(IncDecAddressingMode::ZeroPageX),
    0xF7 => ISC_ZPX => ISC(StoreAddressingMode::ZeroPageX),
    0xF8 => SED     => SED,
    0xF9 => SBC_ABY => SBC(FlexibleAddressingMode::AbsoluteY),
    0xFA => NOP,
    0xFB => ISC_ABY => ISC(StoreAddressingMode::AbsoluteY),
    0xFC => IGN_ABX,
    0xFD => SBC_ABX => SBC(FlexibleAddressingMode::AbsoluteX),
    0xFE => INC_ABX => INC(IncDecAddressingMode::AbsoluteX),
    0xFF => ISC_ABX => ISC(StoreAddressingMode::AbsoluteX),
}
