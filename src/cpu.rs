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

impl CPU {
    fn run_instruction(&mut self) {
        use self::Instruction::*;

        let data = self.fetch();
        let opcode = OpCode::from_u8(data).expect("Unrecognised opcode");
        let instr = opcode.instruction();

        match instr {
            ADC => {
                let value = self.fetch_by(opcode.addressing_mode());

                let (result, carry) = self.accumulator.overflowing_add(value);

                // Perform the operation again, but signed, to check for signed overflow
                self.status.overflow = (self.accumulator as i8).overflowing_add(value as i8).1;

                self.accumulator = result;
                self.status.carry = carry;
            }
            AND => {
                let value = self.fetch_by(opcode.addressing_mode());

                self.accumulator &= value;
            }
            ASL => {
                let value = self.fetch_by(opcode.addressing_mode());

                self.status.carry = value >= 0b10000000;
                self.accumulator = value << 1;
            }
            BCC => self.branch_if(!self.status.carry),
            BCS => self.branch_if(self.status.carry),
            BEQ => self.branch_if(self.status.zero),
            _ => unimplemented!("{:?}", instr),
        }

        self.status.zero = self.accumulator == 0;
        self.status.negative = (self.accumulator as i8).is_negative();
    }

    fn branch_if(&mut self, cond: bool) {
        let offset = self.fetch() as i8;
        if cond {
            self.program_counter += offset;
        }
    }

    fn fetch_by(&mut self, addressing_mode: AddressingMode) -> u8 {
        match addressing_mode {
            AddressingMode::Implied | AddressingMode::Relative => {
                panic!("{:?} does not provide a value", addressing_mode)
            }
            AddressingMode::Immediate => self.fetch(),
            AddressingMode::Accumulator => self.accumulator,
            AddressingMode::Absolute => {
                let lower = self.fetch();
                let higher = self.fetch();
                let address = Address::from_bytes(lower, higher);
                self.deref_address(address)
            }
            _ => unimplemented!("{:?}", addressing_mode),
        }
    }

    fn deref_address(&self, address: Address) -> u8 {
        self.memory[address.0 as usize]
    }

    fn fetch(&mut self) -> u8 {
        let value = self.deref_address(self.program_counter);
        self.program_counter += 1u16;
        value
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
    /// A,Z,C,N = A+M+C
    ///
    /// This instruction adds the contents of a memory location to the accumulator together with the
    /// carry bit. If overflow occurs the carry bit is set, this enables multiple byte addition to
    /// be performed.
    ADC,

    /// A,Z,N = A&M
    ///
    /// A logical AND is performed, bit by bit, on the accumulator contents using the contents of a
    /// byte of memory.
    AND,

    /// A,Z,C,N = M*2 or M,Z,C,N = M*2
    ///
    /// This operation shifts all the bits of the accumulator or memory contents one bit left. Bit 0
    /// is set to 0 and bit 7 is placed in the carry flag. The effect of this operation is to
    /// multiply the memory contents by 2 (ignoring 2's complement considerations), setting the
    /// carry if the result will not fit in 8 bits.
    ASL,

    /// If the carry flag is clear then add the relative displacement to the program counter to
    /// cause a branch to a new location.
    BCC,

    /// If the carry flag is set then add the relative displacement to the program counter to cause
    /// a branch to a new location.
    BCS,

    /// If the zero flag is set then add the relative displacement to the program counter to cause a
    /// branch to a new location.
    BEQ,

    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
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
    fn can_exec_adc_without_overflow() {
        let cpu = run_instr(mem!(ADCImmediate, 10u8), |cpu| {
            cpu.accumulator = 42;
        });

        assert_eq!(cpu.accumulator, 52);
        assert_eq!(cpu.status.overflow, false);
        assert_eq!(cpu.status.carry, false);
    }

    #[test]
    fn can_exec_adc_with_unsigned_overflow() {
        let cpu = run_instr(mem!(ADCImmediate, 255u8), |cpu| {
            cpu.accumulator = 42;
        });

        assert_eq!(cpu.accumulator, 41);
        assert_eq!(cpu.status.overflow, false);
        assert_eq!(cpu.status.carry, true);
    }

    #[test]
    fn can_exec_adc_with_signed_overflow() {
        let cpu = run_instr(mem!(ADCImmediate, 127i8), |cpu| {
            cpu.accumulator = 42i8 as u8;
        });

        assert_eq!(cpu.accumulator as i8, -87i8);
        assert_eq!(cpu.status.overflow, true);
        assert_eq!(cpu.status.carry, false);
    }

    #[test]
    fn can_exec_and() {
        let cpu = run_instr(mem!(ANDImmediate, 0b1100u8), |cpu| {
            cpu.accumulator = 0b1010;
        });

        assert_eq!(cpu.accumulator, 0b1000);
    }

    #[test]
    fn can_exec_asl_without_carry() {
        let cpu = run_instr(mem!(ASLAccumulator), |cpu| {
            cpu.accumulator = 0b100;
        });

        assert_eq!(cpu.accumulator, 0b1000);
        assert_eq!(cpu.status.carry, false);
    }

    #[test]
    fn can_exec_asl_with_carry() {
        let cpu = run_instr(mem!(ASLAccumulator), |cpu| {
            cpu.accumulator = 0b10101010;
        });

        assert_eq!(cpu.accumulator, 0b01010100);
        assert_eq!(cpu.status.carry, true);
    }

    #[test]
    fn can_exec_bcc_when_carry_clear() {
        let cpu = run_instr(mem!(BCC, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.carry = false;
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter, Address(82));
    }

    #[test]
    fn can_exec_bcc_when_carry_set() {
        let cpu = run_instr(mem!(BCC, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.carry = true;
        });

        assert_eq!(cpu.program_counter, Address(92));
    }

    #[test]
    fn can_exec_bcs_when_carry_clear() {
        let cpu = run_instr(mem!(BCS, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.carry = false;
        });

        assert_eq!(cpu.program_counter, Address(92));
    }

    #[test]
    fn can_exec_bcs_when_carry_set() {
        let cpu = run_instr(mem!(BCS, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.carry = true;
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter, Address(82));
    }

    #[test]
    fn can_exec_beq_when_zero_not_set() {
        let cpu = run_instr(mem!(BEQ, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.zero = false;
        });

        assert_eq!(cpu.program_counter, Address(92));
    }

    #[test]
    fn can_exec_beq_when_zero_set() {
        let cpu = run_instr(mem!(BEQ, -10i8), |cpu| {
            cpu.program_counter = Address(90);
            cpu.status.zero = true;
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter, Address(82));
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
        assert_eq!(cpu.fetch_by(AddressingMode::Immediate), 56);
    }

    #[test]
    fn accumulator_addressing_mode_fetches_accumulator_value() {
        let mut cpu = CPU::default();
        cpu.accumulator = 76;
        assert_eq!(cpu.fetch_by(AddressingMode::Accumulator), 76);
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
        assert_eq!(cpu.fetch_by(AddressingMode::Absolute), 35);
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
    }
}
