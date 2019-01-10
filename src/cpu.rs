use crate::opcodes::OpCode;
use num_traits::FromPrimitive;
use num_traits::ToPrimitive;
use std::ops::Add;
use std::ops::AddAssign;

pub trait SerializeBytes {
    fn bytes(self) -> Vec<u8>;
}

impl SerializeBytes for i8 {
    fn bytes(self) -> Vec<u8> {
        vec![self as u8]
    }
}

impl SerializeBytes for u8 {
    fn bytes(self) -> Vec<u8> {
        vec![self]
    }
}

impl SerializeBytes for OpCode {
    fn bytes(self) -> Vec<u8> {
        vec![self as u8]
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct Address(u16);

impl Address {
    fn from_bytes(lower: u8, higher: u8) -> Self {
        Address((u16::from(higher) << 8) + u16::from(lower))
    }

    fn split(self) -> (u8, u8) {
        (self.0 as u8, (self.0 >> 8) as u8)
    }
}

impl SerializeBytes for Address {
    fn bytes(self) -> Vec<u8> {
        let (lower, higher) = self.split();
        vec![lower, higher]
    }
}

impl AddAssign<i8> for Address {
    fn add_assign(&mut self, rhs: i8) {
        self.0 = self.0.wrapping_add(rhs as u16);
    }
}

impl AddAssign<u16> for Address {
    fn add_assign(&mut self, rhs: u16) {
        self.0 = self.0.wrapping_add(rhs);
    }
}

impl Add<u16> for Address {
    type Output = Address;

    fn add(self, rhs: u16) -> <Self as Add<u16>>::Output {
        Address(self.0 + rhs)
    }
}

struct CPU {
    /// 2KB of internal RAM
    memory: [u8; 0x7ff],
    /// A
    accumulator: u8,
    /// X
    x: u8,
    /// Y
    y: u8,
    /// PC
    program_counter: Address,
    /// S
    stack_pointer: u8,
    /// P
    status: Status,
}

fn bit6(value: u8) -> bool {
    value & (1 << 6) != 0
}

fn bit7(value: u8) -> bool {
    value & (1 << 7) != 0
}

impl CPU {
    fn run_instruction(&mut self) {
        use self::Instruction::*;

        let data = *self.fetch();
        let opcode = OpCode::from_u8(data).expect("Unrecognised opcode");
        let instr = opcode.instruction();

        match instr {
            ADC => {
                let value = *self.fetch_by(opcode.addressing_mode());

                let (result, carry) = self.accumulator.overflowing_add(value);

                // Perform the operation again, but signed, to check for signed overflow
                self.status.overflow = (self.accumulator as i8).overflowing_add(value as i8).1;

                self.set_accumulator(result);
                self.status.carry = carry;
            }
            AND => {
                let value = *self.fetch_by(opcode.addressing_mode());

                self.set_accumulator(self.accumulator & value);
            }
            ASL => {
                let addr = self.fetch_by(opcode.addressing_mode());

                let old_value = *addr;
                *addr <<= 1;
                let new_value = *addr;

                self.status.carry = bit7(old_value);
                self.set_flags(new_value);
            }
            BCC => self.branch_if(!self.status.carry),
            BCS => self.branch_if(self.status.carry),
            BEQ => self.branch_if(self.status.zero),
            BIT => {
                let value = *self.fetch_by(opcode.addressing_mode());
                self.status.zero = (self.accumulator & value) == 0;
                self.status.overflow = bit6(value);
                self.status.negative = bit7(value);
            }
            BMI => self.branch_if(self.status.negative),
            BNE => self.branch_if(!self.status.zero),
            BPL => self.branch_if(!self.status.negative),
            BRK => unimplemented!("BRK"), // TODO
            BVC => self.branch_if(!self.status.overflow),
            BVS => self.branch_if(self.status.overflow),
            CLC => self.status.carry = false,
            CLD => self.status.decimal = false,
            CLI => self.status.interrupt_disable = false,
            CLV => self.status.overflow = false,
            CMP => self.compare(self.accumulator, opcode),
            CPX => self.compare(self.x, opcode),
            CPY => self.compare(self.y, opcode),
            DEC => {
                let addr = self.fetch_by(opcode.addressing_mode());
                let (value, carry) = addr.overflowing_sub(1);
                *addr = value;
                self.set_flags(value);
            },
            _ => unimplemented!("{:?}", instr),
        }
    }

    fn compare(&mut self, register: u8, opcode: OpCode) {
        let value = *self.fetch_by(opcode.addressing_mode());
        let (result, carry) = register.overflowing_sub(value);
        self.status.carry = !carry;
        self.set_flags(result);
    }

    fn set_flags(&mut self, value: u8) {
        self.status.zero = value == 0;
        self.status.negative = bit7(value);
    }

    fn set_accumulator(&mut self, value: u8) {
        self.accumulator = value;
        self.set_flags(value);
    }

    fn branch_if(&mut self, cond: bool) {
        let offset = *self.fetch() as i8;
        if cond {
            self.program_counter += offset;
        }
    }

    fn fetch_by(&mut self, addressing_mode: AddressingMode) -> &mut u8 {
        match addressing_mode {
            AddressingMode::Implied | AddressingMode::Relative => {
                panic!("{:?} does not provide a value", addressing_mode)
            }
            AddressingMode::Immediate => self.fetch(),
            AddressingMode::Accumulator => &mut self.accumulator,
            AddressingMode::Absolute => {
                let lower = *self.fetch();
                let higher = *self.fetch();
                let address = Address::from_bytes(lower, higher);
                self.deref_address(address)
            }
            _ => unimplemented!("{:?}", addressing_mode),
        }
    }

    fn deref_address(&mut self, address: Address) -> &mut u8 {
        &mut self.memory[address.0 as usize]
    }

    fn fetch(&mut self) -> &mut u8 {
        let old_program_counter = self.program_counter;
        self.program_counter += 1u16;
        self.deref_address(old_program_counter)
    }
}

impl Default for CPU {
    fn default() -> Self {
        CPU {
            memory: [42; 0x7ff],
            accumulator: 0,
            x: 0,
            y: 0,
            program_counter: Address(0x34),
            stack_pointer: 0xFD,
            status: Status {
                negative: false,
                overflow: false,
                decimal: false,
                interrupt_disable: false,
                zero: false,
                carry: false,
            },
        }
    }
}

struct Status {
    negative: bool,
    overflow: bool,
    decimal: bool,
    interrupt_disable: bool,
    zero: bool,
    carry: bool,
}

#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    /// Add With Carry
    ///
    /// A,Z,C,N = A+M+C
    ///
    /// This instruction adds the contents of a memory location to the accumulator together with the
    /// carry bit. If overflow occurs the carry bit is set, this enables multiple byte addition to
    /// be performed.
    ADC,

    /// Logical AND
    ///
    /// A,Z,N = A&M
    ///
    /// A logical AND is performed, bit by bit, on the accumulator contents using the contents of a
    /// byte of memory.
    AND,

    /// Arithmetic Shift Left
    ///
    /// A,Z,C,N = M*2 or M,Z,C,N = M*2
    ///
    /// This operation shifts all the bits of the accumulator or memory contents one bit left. Bit 0
    /// is set to 0 and bit 7 is placed in the carry flag. The effect of this operation is to
    /// multiply the memory contents by 2 (ignoring 2's complement considerations), setting the
    /// carry if the result will not fit in 8 bits.
    ASL,

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
    BIT,

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
    CMP,

    /// Compare X Register
    ///
    /// Z,C,N = X-M
    ///
    /// This instruction compares the contents of the X register with another memory held value and
    /// sets the zero and carry flags as appropriate.
    CPX,

    /// Compare Y Register
    ///
    ///Z,C,N = Y-M
    ///
    /// This instruction compares the contents of the Y register with another memory held value and
    /// sets the zero and carry flags as appropriate.
    CPY,

    /// Decrement Memory
    /// M,Z,N = M-1
    ///
    /// Subtracts one from the value held at a specified memory location setting the zero and
    /// negative flags as appropriate.
    DEC,
    
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
}

#[derive(Debug, Copy, Clone)]
pub enum AddressingMode {
    Implied,
    Immediate,
    Accumulator,
    Relative,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndexedIndirect,
    IndirectIndexed,
}

#[cfg(test)]
mod tests {
    use super::OpCode::*;
    use super::*;
    use crate::opcodes::OpCode;

    macro_rules! mem {
        ($( $data: expr ),*) => {
            {
                let mut vec: Vec<u8> = vec![];
                $(vec.extend(SerializeBytes::bytes($data));)*
                vec
            }
        };
    }

    #[test]
    fn default_cpu_is_in_default_state() {
        let cpu = CPU::default();

        assert_eq!(cpu.program_counter, Address(0x34));
        assert_eq!(cpu.accumulator, 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.stack_pointer, 0xFD);
    }

    #[test]
    fn instr_adc_adds_numbers() {
        let cpu = run_instr(mem!(ADCImmediate, 10u8), |cpu| {
            cpu.accumulator = 42;
        });

        assert_eq!(cpu.accumulator, 52);
        assert_eq!(cpu.status.overflow, false);
        assert_eq!(cpu.status.carry, false);
    }

    #[test]
    fn instr_adc_sets_carry_flag_on_unsigned_overflow() {
        let cpu = run_instr(mem!(ADCImmediate, 255u8), |cpu| {
            cpu.accumulator = 42;
        });

        assert_eq!(cpu.accumulator, 41);
        assert_eq!(cpu.status.overflow, false);
        assert_eq!(cpu.status.carry, true);
    }

    #[test]
    fn instr_adc_sets_overflow_flag_on_signed_overflow() {
        let cpu = run_instr(mem!(ADCImmediate, 127i8), |cpu| {
            cpu.accumulator = 42i8 as u8;
        });

        assert_eq!(cpu.accumulator as i8, -87i8);
        assert_eq!(cpu.status.overflow, true);
        assert_eq!(cpu.status.carry, false);
    }

    #[test]
    fn instr_and_performs_bitwise_and() {
        let cpu = run_instr(mem!(ANDImmediate, 0b1100u8), |cpu| {
            cpu.accumulator = 0b1010;
        });

        assert_eq!(cpu.accumulator, 0b1000);
    }

    #[test]
    fn instr_asl_shifts_left() {
        let cpu = run_instr(mem!(ASLAccumulator), |cpu| {
            cpu.accumulator = 0b100;
        });

        assert_eq!(cpu.accumulator, 0b1000);
        assert_eq!(cpu.status.carry, false);
    }

    #[test]
    fn instr_asl_sets_carry_flag_on_overflow() {
        let cpu = run_instr(mem!(ASLAccumulator), |cpu| {
            cpu.accumulator = 0b10101010;
        });

        assert_eq!(cpu.accumulator, 0b01010100);
        assert_eq!(cpu.status.carry, true);
    }

    #[test]
    fn instr_asl_can_operate_on_memory() {
        let cpu = run_instr(mem!(ASLAbsolute, Address(100)), |cpu| {
            cpu.set(Address(100), 0b100);
        });

        assert_eq!(cpu.get(Address(100)), 0b1000);
    }

    #[test]
    fn instr_bcc_branches_when_carry_flag_clear() {
        let cpu = run_instr(mem!(BCC, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.carry = false;
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter, Address(82));
    }

    #[test]
    fn instr_bcc_does_not_branch_when_carry_flag_set() {
        let cpu = run_instr(mem!(BCC, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.carry = true;
        });

        assert_eq!(cpu.program_counter, Address(92));
    }

    #[test]
    fn instr_bcs_does_not_branch_when_carry_flag_clear() {
        let cpu = run_instr(mem!(BCS, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.carry = false;
        });

        assert_eq!(cpu.program_counter, Address(92));
    }

    #[test]
    fn instr_bcs_branches_when_carry_flag_set() {
        let cpu = run_instr(mem!(BCS, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.carry = true;
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter, Address(82));
    }

    #[test]
    fn instr_beq_does_not_branch_when_zero_flag_clear() {
        let cpu = run_instr(mem!(BEQ, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.zero = false;
        });

        assert_eq!(cpu.program_counter, Address(92));
    }

    #[test]
    fn instr_beq_branches_when_zero_flag_set() {
        let cpu = run_instr(mem!(BEQ, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.zero = true;
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter, Address(82));
    }

    #[test]
    fn instr_bit_sets_zero_flag_when_bitwise_and_is_zero() {
        let cpu = run_instr(mem!(BITAbsolute, Address(654)), |cpu| {
            cpu.accumulator = 0b11110000u8;
            cpu.set(Address(654), 0b00001111u8);
        });

        assert_eq!(cpu.status.zero, true);
    }

    #[test]
    fn instr_bit_clears_zero_flag_when_bitwise_and_is_not_zero() {
        let cpu = run_instr(mem!(BITAbsolute, Address(654)), |cpu| {
            cpu.accumulator = 0b11111100u8;
            cpu.set(Address(654), 0b00111111u8);
        });

        assert_eq!(cpu.status.zero, false);
    }

    #[test]
    fn instr_bit_sets_overflow_bit_based_on_bit_6_of_operand() {
        let cpu = run_instr(mem!(BITAbsolute, Address(654)), |cpu| {
            cpu.set(Address(654), 0u8);
        });

        assert_eq!(cpu.status.overflow, false);

        let cpu = run_instr(mem!(BITAbsolute, Address(654)), |cpu| {
            cpu.set(Address(654), 0b01000000u8);
        });

        assert_eq!(cpu.status.overflow, true);
    }

    #[test]
    fn instr_bit_sets_negative_bit_based_on_bit_7_of_operand() {
        let cpu = run_instr(mem!(BITAbsolute, Address(654)), |cpu| {
            cpu.set(Address(654), 0u8);
        });

        assert_eq!(cpu.status.negative, false);

        let cpu = run_instr(mem!(BITAbsolute, Address(654)), |cpu| {
            cpu.set(Address(654), 0b10000000u8);
        });

        assert_eq!(cpu.status.negative, true);
    }

    #[test]
    fn instr_bmi_does_not_branch_when_negative_flag_clear() {
        let cpu = run_instr(mem!(BMI, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.negative = false;
        });

        assert_eq!(cpu.program_counter, Address(92));
    }

    #[test]
    fn instr_bmi_branches_when_negative_flag_set() {
        let cpu = run_instr(mem!(BMI, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.negative = true;
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter, Address(82));
    }

    #[test]
    fn instr_bne_branches_when_zero_flag_clear() {
        let cpu = run_instr(mem!(BNE, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.zero = false;
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter, Address(82));
    }

    #[test]
    fn instr_bne_does_not_branch_when_zero_flag_set() {
        let cpu = run_instr(mem!(BNE, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.zero = true;
        });

        assert_eq!(cpu.program_counter, Address(92));
    }

    #[test]
    fn instr_bpl_branches_when_negative_flag_clear() {
        let cpu = run_instr(mem!(BPL, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.negative = false;
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter, Address(82));
    }

    #[test]
    fn instr_bpl_does_not_branch_when_negative_flag_set() {
        let cpu = run_instr(mem!(BPL, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.negative = true;
        });

        assert_eq!(cpu.program_counter, Address(92));
    }

    #[test]
    fn instr_bvc_branches_when_overflow_flag_clear() {
        let cpu = run_instr(mem!(BVC, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.overflow = false;
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter, Address(82));
    }

    #[test]
    fn instr_bvc_does_not_branch_when_overflow_flag_set() {
        let cpu = run_instr(mem!(BVC, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.overflow = true;
        });

        assert_eq!(cpu.program_counter, Address(92));
    }

    #[test]
    fn instr_bvs_does_not_branch_when_carry_flag_clear() {
        let cpu = run_instr(mem!(BVS, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.overflow = false;
        });

        assert_eq!(cpu.program_counter, Address(92));
    }

    #[test]
    fn instr_bvs_branches_when_carry_flag_set() {
        let cpu = run_instr(mem!(BVS, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.overflow = true;
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter, Address(82));
    }

    #[test]
    fn instr_clc_clears_carry_flag() {
        let cpu = run_instr(mem!(CLC), |cpu| {
            cpu.status.carry = true;
        });

        assert_eq!(cpu.status.carry, false);
    }

    #[test]
    fn instr_cld_clears_decimal_flag() {
        let cpu = run_instr(mem!(CLD), |cpu| {
            cpu.status.decimal = true;
        });

        assert_eq!(cpu.status.decimal, false);
    }

    #[test]
    fn instr_cli_clears_interrupt_disable_flag() {
        let cpu = run_instr(mem!(CLI), |cpu| {
            cpu.status.interrupt_disable = true;
        });

        assert_eq!(cpu.status.interrupt_disable, false);
    }

    #[test]
    fn instr_clv_clears_overflow_flag() {
        let cpu = run_instr(mem!(CLV), |cpu| {
            cpu.status.overflow = true;
        });

        assert_eq!(cpu.status.overflow, false);
    }
    
    #[test]
    fn instr_cmp_sets_carry_flag_if_accumulator_greater_or_equal_to_operand() {
        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            cpu.accumulator = 1;
        });

        assert_eq!(cpu.status.carry, false);

        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            cpu.accumulator = 10;
        });

        assert_eq!(cpu.status.carry, true);

        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            cpu.accumulator = 100;
        });

        assert_eq!(cpu.status.carry, true);
    }

    #[test]
    fn instr_cmp_sets_zero_flag_if_accumulator_equals_operand() {
        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            cpu.accumulator = 1;
        });

        assert_eq!(cpu.status.zero, false);

        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            cpu.accumulator = 10;
        });

        assert_eq!(cpu.status.zero, true);

        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            cpu.accumulator = 100;
        });

        assert_eq!(cpu.status.zero, false);
    }

    #[test]
    fn instr_cmp_sets_negative_flag_if_bit_7_of_accumulator_sub_operand_is_set() {
        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            cpu.accumulator = 1;
        });

        assert_eq!(cpu.status.negative, true);

        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            cpu.accumulator = 10;
        });

        assert_eq!(cpu.status.negative, false);

        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            cpu.accumulator = 100;
        });

        assert_eq!(cpu.status.negative, false);
    }

    #[test]
    fn instr_cpx_compares_using_x_register() {
        let cpu = run_instr(mem!(CPXImmediate, 10u8), |cpu| {
            cpu.x = 1;
        });

        assert_eq!(cpu.status.carry, false);
        assert_eq!(cpu.status.zero, false);
        assert_eq!(cpu.status.negative, true);

        let cpu = run_instr(mem!(CPXImmediate, 10u8), |cpu| {
            cpu.x = 10;
        });

        assert_eq!(cpu.status.carry, true);
        assert_eq!(cpu.status.zero, true);
        assert_eq!(cpu.status.negative, false);

        let cpu = run_instr(mem!(CPXImmediate, 10u8), |cpu| {
            cpu.x = 100;
        });

        assert_eq!(cpu.status.carry, true);
        assert_eq!(cpu.status.zero, false);
        assert_eq!(cpu.status.negative, false);
    }

    #[test]
    fn instr_cpy_compares_using_y_register() {
        let cpu = run_instr(mem!(CPYImmediate, 10u8), |cpu| {
            cpu.y = 1;
        });

        assert_eq!(cpu.status.carry, false);
        assert_eq!(cpu.status.zero, false);
        assert_eq!(cpu.status.negative, true);

        let cpu = run_instr(mem!(CPYImmediate, 10u8), |cpu| {
            cpu.y = 10;
        });

        assert_eq!(cpu.status.carry, true);
        assert_eq!(cpu.status.zero, true);
        assert_eq!(cpu.status.negative, false);

        let cpu = run_instr(mem!(CPYImmediate, 10u8), |cpu| {
            cpu.y = 100;
        });

        assert_eq!(cpu.status.carry, true);
        assert_eq!(cpu.status.zero, false);
        assert_eq!(cpu.status.negative, false);
    }
    
    #[test]
    fn instr_dec_decrements_operand() {
        let cpu = run_instr(mem!(DECAbsolute, Address(100)), |cpu| {
            cpu.set(Address(100), 45);
        });

        assert_eq!(cpu.get(Address(100)), 44);
    }

    #[test]
    fn instr_dec_sets_zero_flag_based_on_result() {
        let cpu = run_instr(mem!(DECAbsolute, Address(100)), |cpu| {
            cpu.set(Address(100), 45);
        });

        assert_eq!(cpu.get(Address(100)), 44);
        assert_eq!(cpu.status.zero, false);

        let cpu = run_instr(mem!(DECAbsolute, Address(100)), |cpu| {
            cpu.set(Address(100), 1);
        });

        assert_eq!(cpu.get(Address(100)), 0);
        assert_eq!(cpu.status.zero, true);
    }

    #[test]
    fn instr_dec_sets_negative_flag_based_on_result() {
        let cpu = run_instr(mem!(DECAbsolute, Address(100)), |cpu| {
            cpu.set(Address(100), 45);
        });

        assert_eq!(cpu.get(Address(100)), 44);
        assert_eq!(cpu.status.zero, false);

        let cpu = run_instr(mem!(DECAbsolute, Address(100)), |cpu| {
            cpu.set(Address(100), 0);
        });

        assert_eq!(cpu.get(Address(100)) as i8, -1i8);
        assert_eq!(cpu.status.negative, true);
    }

    #[test]
    #[should_panic]
    fn implied_addressing_mode_does_not_fetch() {
        let mut cpu = CPU::default();
        cpu.fetch_by(AddressingMode::Implied);
    }

    #[test]
    fn immediate_addressing_mode_fetches_given_value() {
        let mut cpu = CPU::default();
        cpu.set(cpu.program_counter, 56);
        assert_eq!(*cpu.fetch_by(AddressingMode::Immediate), 56);
    }

    #[test]
    fn accumulator_addressing_mode_fetches_accumulator_value() {
        let mut cpu = CPU::default();
        cpu.accumulator = 76;
        assert_eq!(*cpu.fetch_by(AddressingMode::Accumulator), 76);
    }

    #[test]
    #[should_panic]
    fn relative_addressing_mode_does_not_fetch() {
        let mut cpu = CPU::default();
        cpu.fetch_by(AddressingMode::Relative);
    }

    #[test]
    fn absolute_addressing_mode_fetches_values_at_given_address() {
        let mut cpu = CPU::default();
        let (lower, higher) = Address(432).split();
        cpu.set(cpu.program_counter, lower);
        cpu.set(cpu.program_counter + 1u16, higher);
        cpu.set(Address(432), 35);
        assert_eq!(*cpu.fetch_by(AddressingMode::Absolute), 35);
    }

    #[test]
    fn zero_flag_is_not_set_when_accumulator_is_non_zero() {
        let cpu = run_instr(mem!(ADCImmediate, 1u8), |cpu| {
            cpu.accumulator = 42;
        });

        assert_eq!(cpu.accumulator, 43);
        assert_eq!(cpu.status.zero, false);
    }

    #[test]
    fn zero_flag_is_set_when_accumulator_is_zero() {
        let cpu = run_instr(mem!(ADCImmediate, 214u8), |cpu| {
            cpu.accumulator = 42;
        });

        assert_eq!(cpu.accumulator, 0);
        assert_eq!(cpu.status.zero, true);
    }

    #[test]
    fn negative_flag_is_not_set_when_accumulator_is_positive() {
        let cpu = run_instr(mem!(ADCImmediate, 1u8), |cpu| {
            cpu.accumulator = 42;
        });

        assert_eq!(cpu.accumulator, 43);
        assert_eq!(cpu.status.negative, false);
    }

    #[test]
    fn negative_flag_is_set_when_accumulator_is_negative() {
        let cpu = run_instr(mem!(ADCImmediate, -1i8), |cpu| {
            cpu.accumulator = 0;
        });

        assert_eq!(cpu.accumulator as i8, -1i8);
        assert_eq!(cpu.status.negative, true);
    }

    #[test]
    fn program_counter_is_incremented_by_1_when_executing_1_byte_instr() {
        let cpu = run_instr(mem!(ASLAccumulator), |cpu| {
            cpu.program_counter = Address(100)
        });

        assert_eq!(cpu.program_counter, Address(101));
    }

    #[test]
    fn program_counter_is_incremented_by_2_when_executing_2_byte_instr() {
        let cpu = run_instr(mem!(ADCImmediate, 0u8), |cpu| {
            cpu.program_counter = Address(100)
        });

        assert_eq!(cpu.program_counter, Address(102));
    }

    #[test]
    fn program_counter_is_incremented_by_3_when_executing_3_byte_instr() {
        let cpu = run_instr(mem!(ASLAbsolute, Address(0)), |cpu| {
            cpu.program_counter = Address(100)
        });

        assert_eq!(cpu.program_counter, Address(103));
    }

    fn run_instr<F: FnOnce(&mut CPU)>(data: Vec<u8>, cpu_setup: F) -> CPU {
        let mut cpu = CPU::default();

        cpu_setup(&mut cpu);

        let mut pc = cpu.program_counter;

        for byte in data.iter() {
            cpu.set(pc, *byte);
            pc += 1u16;
        }

        println!("Loaded data: {:#?}", data);

        cpu.run_instruction();

        cpu
    }

    impl CPU {
        fn set(&mut self, address: Address, byte: u8) {
            self.memory[address.0 as usize] = byte;
        }

        fn get(&self, address: Address) -> u8 {
            self.memory[address.0 as usize]
        }
    }
}
