use crate::addressing_modes::BITAddressingMode;
use crate::addressing_modes::CompareAddressingMode;
use crate::addressing_modes::FlexibleAddressingMode;
use crate::addressing_modes::IncDecAddressingMode;
use crate::addressing_modes::JumpAddressingMode;
use crate::addressing_modes::LDXAddressingMode;
use crate::addressing_modes::LDYAddressingMode;
use crate::addressing_modes::STXAddressingMode;
use crate::addressing_modes::STYAddressingMode;
use crate::addressing_modes::ShiftAddressingMode;
use crate::addressing_modes::StoreAddressingMode;

#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    /// Add With Carry
    ///
    /// A,Z,C,N = A+M+C
    ///
    /// This instruction adds the contents of a memory location to the accumulator together with the
    /// carry bit. If overflow occurs the carry bit is set, this enables multiple byte addition to
    /// be performed.
    ADC(FlexibleAddressingMode),

    /// Logical AND
    ///
    /// A,Z,N = A&M
    ///
    /// A logical AND is performed, bit by bit, on the accumulator contents using the contents of a
    /// byte of memory.
    AND(FlexibleAddressingMode),

    /// Arithmetic Shift Left
    ///
    /// A,Z,C,N = M*2 or M,Z,C,N = M*2
    ///
    /// This operation shifts all the bits of the accumulator or memory contents one bit left. Bit 0
    /// is set to 0 and bit 7 is placed in the carry flag. The effect of this operation is to
    /// multiply the memory contents by 2 (ignoring 2's complement considerations), setting the
    /// carry if the result will not fit in 8 bits.
    ASL(ShiftAddressingMode),

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

    /// Bit Test
    ///
    /// A & M, N = M7, V = M6
    ///
    /// This instructions is used to test if one or more bits are set in a target memory location.
    /// The mask pattern in A is ANDed with the value in memory to set or clear the zero flag, but
    /// the result is not kept. Bits 7 and 6 of the value from memory are copied into the N and V
    /// flags.
    BIT(BITAddressingMode),

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

    /// Force Interrupt
    ///
    /// The BRK instruction forces the generation of an interrupt request. The program counter and
    /// processor status are pushed on the stack then the IRQ interrupt vector at $FFFE/F is loaded
    /// into the PC and the break flag in the status set to one.
    BRK,

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

    /// Exclusive OR
    ///
    /// A,Z,N = A^M
    ///
    /// An exclusive OR is performed, bit by bit, on the accumulator contents using the contents of
    /// a byte of memory.
    EOR(FlexibleAddressingMode),

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

    /// Jump
    ///
    /// Sets the program counter to the address specified by the operand.
    JMP(JumpAddressingMode),

    /// Jump to Subroutine
    ///
    /// The JSR instruction pushes the address (minus one) of the return point on to the stack and
    /// then sets the program counter to the target memory address.
    JSR,

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

    /// Logical Shift Right
    ///
    /// A,C,Z,N = A/2 or M,C,Z,N = M/2
    ///
    /// Each of the bits in A or M is shift one place to the right. The bit that was in bit 0 is
    /// shifted into the carry flag. Bit 7 is set to zero.
    LSR(ShiftAddressingMode),

    /// No Operation
    ///
    /// The NOP instruction causes no changes to the processor other than the normal incrementing of
    /// the program counter to the next instruction.
    NOP,

    ORA(FlexibleAddressingMode),
    PHA,
    PHP,
    PLA,
    PLP,
    ROL(ShiftAddressingMode),
    ROR(ShiftAddressingMode),
    RTI,
    RTS,
    SBC(FlexibleAddressingMode),
    SEC,
    SED,
    SEI,
    STA(StoreAddressingMode),
    STX(STXAddressingMode),
    STY(STYAddressingMode),
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
}
