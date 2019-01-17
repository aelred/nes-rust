use std::fmt;

use log::trace;

use crate::address::Address;
use crate::memory::Memory;

use self::addressing_modes::ShiftAddressingMode;
pub use self::instruction::Instruction;
pub use self::instruction::instructions;

mod addressing_modes;
mod instruction;

const STACK: Address = Address::new(0x0100);
const RESET_VECTOR: Address = Address::new(0xFFFC);
const INTERRUPT_VECTOR: Address = Address::new(0xFFFE);

pub struct CPU<M> {
    /// 2KB of internal RAM, plus more mapped space
    memory: M,
    /// A
    accumulator: u8,
    /// PC
    program_counter: Address,
    /// X
    x: u8,
    /// Y
    y: u8,
    /// S
    stack_pointer: u8,
    /// P
    status: Status,
}

impl<M: Memory> CPU<M> {
    pub fn with_memory(memory: M) -> Self {
        let lower = memory.read(RESET_VECTOR);
        let higher = memory.read(RESET_VECTOR + 1);
        let program_counter = Address::from_bytes(higher, lower);

        CPU {
            memory,
            accumulator: 0,
            program_counter,
            x: 0,
            y: 0,
            stack_pointer: 0xFF,
            status: Status(0),
        }
    }

    pub fn read(&self, address: Address) -> u8 {
        self.read_reference(Reference::Address(address))
    }

    fn read_address(&self, address: Address) -> Address {
        let lower = self.read(address);
        let higher = self.read(address.incr_lower());
        Address::from_bytes(higher, lower)
    }

    pub fn write(&mut self, address: Address, byte: u8) {
        self.write_reference(Reference::Address(address), byte);
    }

    pub fn set_program_counter(&mut self, address: Address) {
        self.program_counter = address;
    }

    fn accumulator(&self) -> u8 {
        self.accumulator
    }

    pub fn run_instruction(&mut self) {
        use self::instruction::Instruction::*;

        match self.instr() {
            // Load/Store Operations
            LDA(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                self.set_accumulator(value);
            }
            LDX(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                self.set_x(value);
            }
            LDY(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                self.set_y(value);
            }
            STA(addressing_mode) => {
                let reference = self.fetch_ref(addressing_mode);
                self.write_reference(reference, self.accumulator());
            }
            STX(addressing_mode) => {
                let reference = self.fetch_ref(addressing_mode);
                self.write_reference(reference, self.x());
            }
            STY(addressing_mode) => {
                let reference = self.fetch_ref(addressing_mode);
                self.write_reference(reference, self.y());
            }

            // Register Transfers
            TAX => {
                self.set_x(self.accumulator());
            }
            TAY => {
                self.set_y(self.accumulator());
            }
            TXA => {
                self.set_accumulator(self.x());
            }
            TYA => {
                self.set_accumulator(self.y());
            }

            // Stack Operations
            TSX => {
                self.set_x(self.stack_pointer);
            }
            TXS => {
                self.stack_pointer = self.x();
            }
            PLA => {
                let accumulator = self.pull_stack();
                self.set_accumulator(accumulator);
            }
            PLP => {
                self.status = Status(self.pull_stack());
            }
            PHA => self.push_stack(self.accumulator()),
            PHP => self.push_stack(self.status.0),

            // Logical
            AND(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                self.set_accumulator(self.accumulator() & value);
            }
            EOR(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                self.set_accumulator(self.accumulator() ^ value);
            }
            ORA(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                self.set_accumulator(self.accumulator() | value);
            }
            BIT(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                let result = self.accumulator() & value;
                self.status.set_to(Flag::Zero, result == 0);
                self.status.set_to(Flag::Overflow, value & (1 << 6) != 0);
                self.status
                    .set_to(Flag::Negative, (value as i8).is_negative());
            }

            // Arithmetic
            ADC(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                self.add_to_accumulator(value);
            }
            SBC(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                self.sub_from_accumulator(value);
            }
            CMP(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                self.compare(self.accumulator(), value);
            },
            CPX(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                self.compare(self.x(), value);
            },
            CPY(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                self.compare(self.y(), value)
            },

            // Increments & Decrements
            INC(addressing_mode) => {
                let reference = self.fetch_ref(addressing_mode);
                self.increment(reference);
            }
            INX => self.increment(Reference::X),
            INY => self.increment(Reference::Y),
            DEC(addressing_mode) => {
                let reference = self.fetch_ref(addressing_mode);
                self.decrement(reference);
            }
            DEX => self.decrement(Reference::X),
            DEY => self.decrement(Reference::Y),

            // Shifts
            ASL(addressing_mode) => {
                let reference = self.fetch_ref(addressing_mode);
                self.asl(reference)
            },
            LSR(addressing_mode) => {
                let reference = self.fetch_ref(addressing_mode);
                self.shift(reference, 0, |val, _| val >> 1)
            },
            ROL(addressing_mode) => {
                let reference = self.fetch_ref(addressing_mode);
                self.rol(reference);
            }
            ROR(addressing_mode) => {
                let reference = self.fetch_ref(addressing_mode);
                self.shift(reference, 0, |val, carry| val >> 1 | carry << 7);
            }

            // Jumps & Calls
            JMP(addressing_mode) => {
                let addr = addressing_mode.fetch_address(self);
                *self.program_counter_mut() = addr;
            }
            JSR => {
                let addr = self.fetch_address_at_program_counter();

                // For some reason the spec says the pointer must be to the last byte of the JSR
                // instruction...
                let data = self.program_counter() - 1;

                self.push_stack(data.higher());
                self.push_stack(data.lower());

                *self.program_counter_mut() = addr;
            }
            RTS => {
                let lower = self.pull_stack();
                let higher = self.pull_stack();
                *self.program_counter_mut() = Address::from_bytes(higher, lower) + 1;
            }

            // Branches
            BCC => self.branch_if(!self.status.get(Flag::Carry)),
            BCS => self.branch_if(self.status.get(Flag::Carry)),
            BEQ => self.branch_if(self.status.get(Flag::Zero)),
            BMI => self.branch_if(self.status.get(Flag::Negative)),
            BNE => self.branch_if(!self.status.get(Flag::Zero)),
            BPL => self.branch_if(!self.status.get(Flag::Negative)),
            BVC => self.branch_if(!self.status.get(Flag::Overflow)),
            BVS => self.branch_if(self.status.get(Flag::Overflow)),

            // Status Flag Changes
            CLC => self.status.clear(Flag::Carry),
            CLD => self.status.clear(Flag::Decimal),
            CLI => self.status.clear(Flag::InterruptDisable),
            CLV => self.status.clear(Flag::Overflow),
            SEC => self.status.set(Flag::Carry),
            SED => self.status.set(Flag::Decimal),
            SEI => self.status.set(Flag::InterruptDisable),

            // System Functions
            BRK => {
                self.status.set(Flag::Break);

                let addr = self.read_address(INTERRUPT_VECTOR);

                // For some reason the spec says the pointer must be to the last byte of the BRK
                // instruction...
                let data = self.program_counter() - 1;

                self.push_stack(data.higher());
                self.push_stack(data.lower());
                self.push_stack(self.status.0);

                *self.program_counter_mut() = addr;
            }
            NOP => {}
            RTI => {
                self.status = Status(self.pull_stack());
                let lower = self.pull_stack();
                let higher = self.pull_stack();
                *self.program_counter_mut() = Address::from_bytes(higher, lower);
            }

            // Unofficial Opcodes
            IGN(addressing_mode) => {
                self.fetch_ref(addressing_mode);
            }
            SKB => {
                self.fetch_at_program_counter();
            }
            LAX(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                self.set_accumulator(value);
                self.set_x(value);
            }
            SAX(addressing_mode) => {
                let reference = self.fetch_ref(addressing_mode);
                self.write_reference(reference, self.accumulator() & self.x());
            }
            DCP(addressing_mode) => {
                let reference = self.fetch_ref(addressing_mode);
                self.decrement(reference);
                self.compare(self.accumulator(), self.read_reference(reference));
            }
            ISC(addressing_mode) => {
                let reference = self.fetch_ref(addressing_mode);
                self.increment(reference);
                self.sub_from_accumulator(self.read_reference(reference));
            }
            SLO(addressing_mode) => {
                let reference = self.fetch_ref(addressing_mode);
                self.asl(reference);
                self.set_accumulator(self.accumulator() | self.read_reference(reference));
            }
            RLA(addressing_mode) => {
                let reference = self.fetch_ref(addressing_mode);
                self.rol(reference);
                self.set_accumulator(self.accumulator() & self.read_reference(reference));
            }
        }
    }

    fn asl(&mut self, reference: Reference) {
        self.shift(reference, 7, |val, _| val << 1);
    }

    fn rol(&mut self, reference: Reference) {
        self.shift(reference, 7, |val, carry| val << 1 | carry);
    }

    fn sub_from_accumulator(&mut self, value: u8) {
        self.add_to_accumulator(!value);
    }

    fn add_to_accumulator(&mut self, value: u8) {
        let accumulator = self.accumulator();

        let carry_in = self.status.get(Flag::Carry) as u16;

        let full_result = u16::from(accumulator)
            .wrapping_add(u16::from(value))
            .wrapping_add(carry_in);

        let result = full_result as u8;
        let carry_out = full_result & (1 << 8) != 0;

        // Check if the sign bit has changed
        let overflow = (((accumulator ^ result) & (value ^ result)) as i8).is_negative();
        self.status.set_to(Flag::Overflow, overflow);

        self.set_accumulator(result);
        self.status.set_to(Flag::Carry, carry_out);
    }

    fn shift(&mut self, reference: Reference, carry_bit: u8, op: impl FnOnce(u8, u8) -> (u8)) {
        let carry = self.status.get(Flag::Carry);

        let old_value = self.read_reference(reference);
        let new_value = op(old_value, carry as u8);
        let carry = old_value & (1 << carry_bit) != 0;

        self.set_reference(reference, new_value);
        self.status.set_to(Flag::Carry, carry);
    }

    fn push_stack(&mut self, byte: u8) {
        let stack_address = STACK + u16::from(self.stack_pointer);
        self.write(stack_address, byte);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn pull_stack(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let stack_address = STACK + u16::from(self.stack_pointer);
        self.read(stack_address)
    }

    fn increment(&mut self, reference: Reference) {
        let value = self.read_reference(reference).wrapping_add(1);
        self.set_reference(reference, value);
    }

    fn decrement(&mut self, reference: Reference) {
        let value = self.read_reference(reference).wrapping_sub(1);
        self.set_reference(reference, value);
    }

    fn program_counter(&self) -> Address {
        self.program_counter
    }

    fn program_counter_mut(&mut self) -> &mut Address {
        &mut self.program_counter
    }

    fn x(&self) -> u8 {
        self.x
    }

    fn y(&self) -> u8 {
        self.y
    }

    fn compare(&mut self, register: u8, value: u8) {
        let (result, carry) = register.overflowing_sub(value);
        self.status.set_to(Flag::Carry, !carry);
        self.status.set_flags(result);
    }

    fn set_reference(&mut self, reference: Reference, value: u8) {
        self.write_reference(reference, value);
        self.status.set_flags(value);
    }

    fn set_accumulator(&mut self, value: u8) {
        self.set_reference(Reference::Accumulator, value);
    }

    fn set_x(&mut self, value: u8) {
        self.set_reference(Reference::X, value);
    }

    fn set_y(&mut self, value: u8) {
        self.set_reference(Reference::Y, value);
    }

    fn branch_if(&mut self, cond: bool) {
        let offset = self.fetch_at_program_counter() as i8;
        if cond {
            *self.program_counter_mut() += offset as u16;
        }
    }

    fn fetch_ref<T: ReferenceAddressingMode>(&mut self, addressing_mode: T) -> Reference {
        addressing_mode.fetch_ref(self)
    }

    fn fetch<T: ReferenceAddressingMode>(&mut self, addressing_mode: T) -> u8 {
        let reference = self.fetch_ref(addressing_mode);
        self.read_reference(reference)
    }

    fn read_reference(&self, reference: Reference) -> u8 {
        match reference {
            Reference::Address(address) => self.memory.read(address),
            Reference::Accumulator => self.accumulator,
            Reference::X => self.x,
            Reference::Y => self.y,
        }
    }

    fn write_reference(&mut self, reference: Reference, byte: u8) {
        trace!("        {} := {:<#04x}", reference, byte);
        match reference {
            Reference::Address(address) => self.memory.write(address, byte),
            Reference::Accumulator => self.accumulator = byte,
            Reference::X => self.x = byte,
            Reference::Y => self.y = byte,
        };
    }

    fn instr(&mut self) -> Instruction {
        let instruction = Instruction::from_opcode(self.fetch_at_program_counter());
        trace!("        {:?}", instruction);
        instruction
    }

    fn fetch_at_program_counter(&mut self) -> u8 {
        let data = self.read(self.program_counter);
        trace!("{}  {:#04x}", self.program_counter, data);
        self.program_counter += 1u16;
        data
    }

    fn fetch_address_at_program_counter(&mut self) -> Address {
        let lower = self.fetch_at_program_counter();
        let higher = self.fetch_at_program_counter();
        Address::from_bytes(higher, lower)
    }
}

trait ReferenceAddressingMode {
    fn fetch_ref<M: Memory>(self, cpu: &mut CPU<M>) -> Reference;
}

#[derive(Copy, Clone)]
enum Reference {
    Address(Address),
    Accumulator,
    X,
    Y,
}

impl fmt::Display for Reference {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        match self {
            Reference::Address(address) => write!(f, "{}", address),
            Reference::Accumulator => f.write_str("A"),
            Reference::X => f.write_str("X"),
            Reference::Y => f.write_str("Y"),
        }
    }
}

#[derive(Copy, Clone)]
struct Status(u8);

impl Status {
    fn get(self, flag: Flag) -> bool {
        (self.0 & flag as u8) != 0
    }

    fn set(&mut self, flag: Flag) {
        self.0 |= flag as u8;
    }

    fn clear(&mut self, flag: Flag) {
        self.0 &= !(flag as u8);
    }

    fn set_to(&mut self, flag: Flag, value: bool) {
        if value {
            self.set(flag);
        } else {
            self.clear(flag);
        }
    }

    fn set_flags(&mut self, value: u8) {
        self.set_to(Flag::Zero, value == 0);
        self.set_to(Flag::Negative, (value as i8).is_negative());
    }
}

impl From<u8> for Status {
    fn from(byte: u8) -> Self {
        Status(byte)
    }
}

impl Into<u8> for Status {
    fn into(self) -> u8 {
        self.0
    }
}

enum Flag {
    Negative = 0b1000_0000,
    Overflow = 0b0100_0000,
    Break = 0b0001_0000,
    Decimal = 0b0000_1000,
    InterruptDisable = 0b0000_0100,
    Zero = 0b0000_0010,
    Carry = 0b0000_0001,
}

#[cfg(test)]
mod tests {
    use crate::ArrayMemory;
    use crate::mem;

    use super::*;
    use super::instructions::*;

    #[test]
    fn cpu_initialises_in_default_state() {
        let cpu = CPU::with_memory(ArrayMemory::default());

        assert_eq!(cpu.program_counter(), Address::new(0x00));
        assert_eq!(cpu.accumulator(), 0);
        assert_eq!(cpu.x(), 0);
        assert_eq!(cpu.y(), 0);
        assert_eq!(cpu.stack_pointer, 0xFF);
    }

    #[test]
    fn cpu_initialises_program_counter_to_reset_vector() {
        let memory = mem! {
            0xFFFC => { 0x34, 0x12 }
        };

        let cpu = CPU::with_memory(memory);

        assert_eq!(cpu.program_counter(), Address::new(0x1234));
    }

    #[test]
    fn instr_adc_adds_numbers() {
        let cpu = run_instr(mem!(ADC_IMMEDIATE, 10u8), |cpu| {
            cpu.set_accumulator(42);
        });

        assert_eq!(cpu.accumulator(), 52);
        assert_eq!(cpu.status.get(Flag::Overflow), false);
        assert_eq!(cpu.status.get(Flag::Carry), false);
    }

    #[test]
    fn instr_adc_sets_carry_flag_on_unsigned_overflow() {
        let cpu = run_instr(mem!(ADC_IMMEDIATE, 255u8), |cpu| {
            cpu.set_accumulator(42);
        });

        assert_eq!(cpu.accumulator(), 41);
        assert_eq!(cpu.status.get(Flag::Overflow), false);
        assert_eq!(cpu.status.get(Flag::Carry), true);
    }

    #[test]
    fn instr_adc_sets_overflow_flag_on_signed_overflow() {
        let cpu = run_instr(mem!(ADC_IMMEDIATE, 127u8), |cpu| {
            cpu.set_accumulator(42i8 as u8);
        });

        assert_eq!(cpu.accumulator() as i8, -87i8);
        assert_eq!(cpu.status.get(Flag::Overflow), true);
        assert_eq!(cpu.status.get(Flag::Carry), false);
    }

    #[test]
    fn instr_and_performs_bitwise_and() {
        let cpu = run_instr(mem!(AND_IMMEDIATE, 0b1100_u8), |cpu| {
            cpu.set_accumulator(0b1010);
        });

        assert_eq!(cpu.accumulator(), 0b1000);
    }

    #[test]
    fn instr_asl_shifts_left() {
        let cpu = run_instr(mem!(ASL_ACCUMULATOR), |cpu| {
            cpu.set_accumulator(0b100);
        });

        assert_eq!(cpu.accumulator(), 0b1000);
        assert_eq!(cpu.status.get(Flag::Carry), false);
    }

    #[test]
    fn instr_asl_sets_carry_flag_on_overflow() {
        let cpu = run_instr(mem!(ASL_ACCUMULATOR), |cpu| {
            cpu.set_accumulator(0b1010_1010);
        });

        assert_eq!(cpu.accumulator(), 0b0101_0100);
        assert_eq!(cpu.status.get(Flag::Carry), true);
    }

    #[test]
    fn instr_asl_can_operate_on_memory() {
        let cpu = run_instr(mem!(ASL_ABSOLUTE, 100, 0), |cpu| {
            cpu.set(Address::new(100), 0b100);
        });

        assert_eq!(cpu.get(Address::new(100)), 0b1000);
    }

    #[test]
    fn instr_bcc_branches_when_carry_flag_clear() {
        let cpu = run_instr(mem!(90 => { BCC, -10i8 as u8 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.clear(Flag::Carry);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_bcc_does_not_branch_when_carry_flag_set() {
        let cpu = run_instr(mem!(90 => { BCC, -10i8 as u8 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.set(Flag::Carry);
        });

        assert_eq!(cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_bcs_does_not_branch_when_carry_flag_clear() {
        let cpu = run_instr(mem!(90 => { BCS, -10i8 as u8 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.clear(Flag::Carry);
        });

        assert_eq!(cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_bcs_branches_when_carry_flag_set() {
        let cpu = run_instr(mem!(90 => { BCS, -10i8 as u8 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.set(Flag::Carry);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_beq_does_not_branch_when_zero_flag_clear() {
        let cpu = run_instr(mem!(90 => { BEQ, -10i8 as u8 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.clear(Flag::Zero);
        });

        assert_eq!(cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_beq_branches_when_zero_flag_set() {
        let cpu = run_instr(mem!(90 => { BEQ, -10i8 as u8 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.set(Flag::Zero);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_bit_sets_zero_flag_when_bitwise_and_is_zero() {
        let cpu = run_instr(mem!(BIT_ABSOLUTE, 54, 0), |cpu| {
            cpu.set_accumulator(0b1111_0000u8);
            cpu.set(Address::new(54), 0b0000_1111u8);
        });

        assert_eq!(cpu.status.get(Flag::Zero), true);
    }

    #[test]
    fn instr_bit_clears_zero_flag_when_bitwise_and_is_not_zero() {
        let cpu = run_instr(mem!(BIT_ABSOLUTE, 54, 0), |cpu| {
            cpu.set_accumulator(0b1111_1100u8);
            cpu.set(Address::new(54), 0b0011_1111u8);
        });

        assert_eq!(cpu.status.get(Flag::Zero), false);
    }

    #[test]
    fn instr_bit_sets_overflow_bit_based_on_bit_6_of_operand() {
        let cpu = run_instr(mem!(BIT_ABSOLUTE, 54, 0), |cpu| {
            cpu.set(Address::new(54), 0u8);
        });

        assert_eq!(cpu.status.get(Flag::Overflow), false);

        let cpu = run_instr(mem!(BIT_ABSOLUTE, 54, 0), |cpu| {
            cpu.set(Address::new(54), 0b0100_0000u8);
        });

        assert_eq!(cpu.status.get(Flag::Overflow), true);
    }

    #[test]
    fn instr_bit_sets_negative_bit_based_on_bit_7_of_operand() {
        let cpu = run_instr(mem!(BIT_ABSOLUTE, 54, 0), |cpu| {
            cpu.set(Address::new(54), 0u8);
        });

        assert_eq!(cpu.status.get(Flag::Negative), false);

        let cpu = run_instr(mem!(BIT_ABSOLUTE, 54, 0), |cpu| {
            cpu.set(Address::new(54), 0b1000_0000u8);
        });

        assert_eq!(cpu.status.get(Flag::Negative), true);
    }

    #[test]
    fn instr_bmi_does_not_branch_when_negative_flag_clear() {
        let cpu = run_instr(mem!(90 => { BMI, -10i8 as u8 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.clear(Flag::Negative);
        });

        assert_eq!(cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_bmi_branches_when_negative_flag_set() {
        let cpu = run_instr(mem!(90 => { BMI, -10i8 as u8 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.set(Flag::Negative);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_bne_branches_when_zero_flag_clear() {
        let cpu = run_instr(mem!(90 => { BNE, -10i8 as u8 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.clear(Flag::Zero);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_bne_does_not_branch_when_zero_flag_set() {
        let cpu = run_instr(mem!(90 => { BNE, -10i8 as u8 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.set(Flag::Zero);
        });

        assert_eq!(cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_bpl_branches_when_negative_flag_clear() {
        let cpu = run_instr(mem!(90 => { BPL, -10i8 as u8 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.clear(Flag::Negative);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_bpl_does_not_branch_when_negative_flag_set() {
        let cpu = run_instr(mem!(90 => { BPL, -10i8 as u8 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.set(Flag::Negative);
        });

        assert_eq!(cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_bvc_branches_when_overflow_flag_clear() {
        let cpu = run_instr(mem!(90 => { BVC, -10i8 as u8 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.clear(Flag::Overflow);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_bvc_does_not_branch_when_overflow_flag_set() {
        let cpu = run_instr(mem!(90 => { BVC, -10i8 as u8 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.set(Flag::Overflow);
        });

        assert_eq!(cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_bvs_does_not_branch_when_carry_flag_clear() {
        let cpu = run_instr(mem!(90 => { BVS, -10i8 as u8 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.clear(Flag::Overflow);
        });

        assert_eq!(cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_bvs_branches_when_carry_flag_set() {
        let cpu = run_instr(mem!(90 => { BVS, -10i8 as u8 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.set(Flag::Overflow);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_clc_clears_carry_flag() {
        let cpu = run_instr(mem!(CLC), |cpu| {
            cpu.status.set(Flag::Carry);
        });

        assert_eq!(cpu.status.get(Flag::Carry), false);
    }

    #[test]
    fn instr_cld_clears_decimal_flag() {
        let cpu = run_instr(mem!(CLD), |cpu| {
            cpu.status.set(Flag::Decimal);
        });

        assert_eq!(cpu.status.get(Flag::Decimal), false);
    }

    #[test]
    fn instr_cli_clears_interrupt_disable_flag() {
        let cpu = run_instr(mem!(CLI), |cpu| {
            cpu.status.set(Flag::InterruptDisable);
        });

        assert_eq!(cpu.status.get(Flag::InterruptDisable), false);
    }

    #[test]
    fn instr_clv_clears_overflow_flag() {
        let cpu = run_instr(mem!(CLV), |cpu| {
            cpu.status.set(Flag::Overflow);
        });

        assert_eq!(cpu.status.get(Flag::Overflow), false);
    }

    #[test]
    fn instr_cmp_sets_carry_flag_if_accumulator_greater_or_equal_to_operand() {
        let cpu = run_instr(mem!(CMP_IMMEDIATE, 10u8), |cpu| {
            cpu.set_accumulator(1);
        });

        assert_eq!(cpu.status.get(Flag::Carry), false);

        let cpu = run_instr(mem!(CMP_IMMEDIATE, 10u8), |cpu| {
            cpu.set_accumulator(10);
        });

        assert_eq!(cpu.status.get(Flag::Carry), true);

        let cpu = run_instr(mem!(CMP_IMMEDIATE, 10u8), |cpu| {
            cpu.set_accumulator(100);
        });

        assert_eq!(cpu.status.get(Flag::Carry), true);
    }

    #[test]
    fn instr_cmp_sets_zero_flag_if_accumulator_equals_operand() {
        let cpu = run_instr(mem!(CMP_IMMEDIATE, 10u8), |cpu| {
            cpu.set_accumulator(1);
        });

        assert_eq!(cpu.status.get(Flag::Zero), false);

        let cpu = run_instr(mem!(CMP_IMMEDIATE, 10u8), |cpu| {
            cpu.set_accumulator(10);
        });

        assert_eq!(cpu.status.get(Flag::Zero), true);

        let cpu = run_instr(mem!(CMP_IMMEDIATE, 10u8), |cpu| {
            cpu.set_accumulator(100);
        });

        assert_eq!(cpu.status.get(Flag::Zero), false);
    }

    #[test]
    fn instr_cmp_sets_negative_flag_if_bit_7_of_accumulator_sub_operand_is_set() {
        let cpu = run_instr(mem!(CMP_IMMEDIATE, 10u8), |cpu| {
            cpu.set_accumulator(1);
        });

        assert_eq!(cpu.status.get(Flag::Negative), true);

        let cpu = run_instr(mem!(CMP_IMMEDIATE, 10u8), |cpu| {
            cpu.set_accumulator(10);
        });

        assert_eq!(cpu.status.get(Flag::Negative), false);

        let cpu = run_instr(mem!(CMP_IMMEDIATE, 10u8), |cpu| {
            cpu.set_accumulator(100);
        });

        assert_eq!(cpu.status.get(Flag::Negative), false);
    }

    #[test]
    fn instr_cpx_compares_using_x_register() {
        let cpu = run_instr(mem!(CPX_IMMEDIATE, 10u8), |cpu| {
            cpu.set_x(1);
        });

        assert_eq!(cpu.status.get(Flag::Carry), false);
        assert_eq!(cpu.status.get(Flag::Zero), false);
        assert_eq!(cpu.status.get(Flag::Negative), true);

        let cpu = run_instr(mem!(CPX_IMMEDIATE, 10u8), |cpu| {
            cpu.set_x(10);
        });

        assert_eq!(cpu.status.get(Flag::Carry), true);
        assert_eq!(cpu.status.get(Flag::Zero), true);
        assert_eq!(cpu.status.get(Flag::Negative), false);

        let cpu = run_instr(mem!(CPX_IMMEDIATE, 10u8), |cpu| {
            cpu.set_x(100);
        });

        assert_eq!(cpu.status.get(Flag::Carry), true);
        assert_eq!(cpu.status.get(Flag::Zero), false);
        assert_eq!(cpu.status.get(Flag::Negative), false);
    }

    #[test]
    fn instr_cpy_compares_using_y_register() {
        let cpu = run_instr(mem!(CPY_IMMEDIATE, 10u8), |cpu| {
            cpu.set_y(1);
        });

        assert_eq!(cpu.status.get(Flag::Carry), false);
        assert_eq!(cpu.status.get(Flag::Zero), false);
        assert_eq!(cpu.status.get(Flag::Negative), true);

        let cpu = run_instr(mem!(CPY_IMMEDIATE, 10u8), |cpu| {
            cpu.set_y(10);
        });

        assert_eq!(cpu.status.get(Flag::Carry), true);
        assert_eq!(cpu.status.get(Flag::Zero), true);
        assert_eq!(cpu.status.get(Flag::Negative), false);

        let cpu = run_instr(mem!(CPY_IMMEDIATE, 10u8), |cpu| {
            cpu.set_y(100);
        });

        assert_eq!(cpu.status.get(Flag::Carry), true);
        assert_eq!(cpu.status.get(Flag::Zero), false);
        assert_eq!(cpu.status.get(Flag::Negative), false);
    }

    #[test]
    fn instr_dec_decrements_operand() {
        let cpu = run_instr(mem!(DEC_ABSOLUTE, 100, 0), |cpu| {
            cpu.set(Address::new(100), 45);
        });

        assert_eq!(cpu.get(Address::new(100)), 44);
    }

    #[test]
    fn instr_dec_sets_zero_flag_based_on_result() {
        let cpu = run_instr(mem!(DEC_ABSOLUTE, 100, 0), |cpu| {
            cpu.set(Address::new(100), 45);
        });

        assert_eq!(cpu.get(Address::new(100)), 44);
        assert_eq!(cpu.status.get(Flag::Zero), false);

        let cpu = run_instr(mem!(DEC_ABSOLUTE, 100, 0), |cpu| {
            cpu.set(Address::new(100), 1);
        });

        assert_eq!(cpu.get(Address::new(100)), 0);
        assert_eq!(cpu.status.get(Flag::Zero), true);
    }

    #[test]
    fn instr_dec_sets_negative_flag_based_on_result() {
        let cpu = run_instr(mem!(DEC_ABSOLUTE, 100, 0), |cpu| {
            cpu.set(Address::new(100), 45);
        });

        assert_eq!(cpu.get(Address::new(100)), 44);
        assert_eq!(cpu.status.get(Flag::Zero), false);

        let cpu = run_instr(mem!(DEC_ABSOLUTE, 100, 0), |cpu| {
            cpu.set(Address::new(100), 0);
        });

        assert_eq!(cpu.get(Address::new(100)) as i8, -1i8);
        assert_eq!(cpu.status.get(Flag::Negative), true);
    }

    #[test]
    fn instr_dex_decrements_x_register() {
        let cpu = run_instr(mem!(DEX), |cpu| {
            cpu.set_x(45);
        });

        assert_eq!(cpu.x(), 44);
    }

    #[test]
    fn instr_dex_sets_zero_flag_based_on_result() {
        let cpu = run_instr(mem!(DEX), |cpu| {
            cpu.set_x(45);
        });

        assert_eq!(cpu.x(), 44);
        assert_eq!(cpu.status.get(Flag::Zero), false);

        let cpu = run_instr(mem!(DEX), |cpu| {
            cpu.set_x(1);
        });

        assert_eq!(cpu.x(), 0);
        assert_eq!(cpu.status.get(Flag::Zero), true);
    }

    #[test]
    fn instr_dex_sets_negative_flag_based_on_result() {
        let cpu = run_instr(mem!(DEX), |cpu| {
            cpu.set_x(45);
        });

        assert_eq!(cpu.x(), 44);
        assert_eq!(cpu.status.get(Flag::Zero), false);

        let cpu = run_instr(mem!(DEX), |cpu| {
            cpu.set_x(0);
        });

        assert_eq!(cpu.x() as i8, -1i8);
        assert_eq!(cpu.status.get(Flag::Negative), true);
    }

    #[test]
    fn instr_dey_decrements_y_register() {
        let cpu = run_instr(mem!(DEY), |cpu| {
            cpu.set_y(45);
        });

        assert_eq!(cpu.y(), 44);
    }

    #[test]
    fn instr_dey_sets_zero_flag_based_on_result() {
        let cpu = run_instr(mem!(DEY), |cpu| {
            cpu.set_y(45);
        });

        assert_eq!(cpu.y(), 44);
        assert_eq!(cpu.status.get(Flag::Zero), false);

        let cpu = run_instr(mem!(DEY), |cpu| {
            cpu.set_y(1);
        });

        assert_eq!(cpu.y(), 0);
        assert_eq!(cpu.status.get(Flag::Zero), true);
    }

    #[test]
    fn instr_dey_sets_negative_flag_based_on_result() {
        let cpu = run_instr(mem!(DEY), |cpu| {
            cpu.set_y(45);
        });

        assert_eq!(cpu.y(), 44);
        assert_eq!(cpu.status.get(Flag::Zero), false);

        let cpu = run_instr(mem!(DEY), |cpu| {
            cpu.set_y(0);
        });

        assert_eq!(cpu.y() as i8, -1i8);
        assert_eq!(cpu.status.get(Flag::Negative), true);
    }

    #[test]
    fn instr_eor_performs_bitwise_xor() {
        let cpu = run_instr(mem!(EOR_IMMEDIATE, 0b1100_u8), |cpu| {
            cpu.set_accumulator(0b1010);
        });

        assert_eq!(cpu.accumulator(), 0b0110);
    }

    #[test]
    fn instr_inc_increments_operand() {
        let cpu = run_instr(mem!(INC_ABSOLUTE, 100, 0), |cpu| {
            cpu.set(Address::new(100), 45);
        });

        assert_eq!(cpu.get(Address::new(100)), 46);
    }

    #[test]
    fn instr_inc_sets_zero_flag_based_on_result() {
        let cpu = run_instr(mem!(INC_ABSOLUTE, 100, 0), |cpu| {
            cpu.set(Address::new(100), 45);
        });

        assert_eq!(cpu.get(Address::new(100)), 46);
        assert_eq!(cpu.status.get(Flag::Zero), false);

        let cpu = run_instr(mem!(INC_ABSOLUTE, 100, 0), |cpu| {
            cpu.set(Address::new(100), -1i8 as u8);
        });

        assert_eq!(cpu.get(Address::new(100)), 0);
        assert_eq!(cpu.status.get(Flag::Zero), true);
    }

    #[test]
    fn instr_inc_sets_negative_flag_based_on_result() {
        let cpu = run_instr(mem!(INC_ABSOLUTE, 100, 0), |cpu| {
            cpu.set(Address::new(100), 45);
        });

        assert_eq!(cpu.get(Address::new(100)), 46);
        assert_eq!(cpu.status.get(Flag::Zero), false);

        let cpu = run_instr(mem!(INC_ABSOLUTE, 100, 0), |cpu| {
            cpu.set(Address::new(100), -10i8 as u8);
        });

        assert_eq!(cpu.get(Address::new(100)) as i8, -9i8);
        assert_eq!(cpu.status.get(Flag::Negative), true);
    }

    #[test]
    fn instr_inx_increments_x_register() {
        let cpu = run_instr(mem!(INX), |cpu| {
            cpu.set_x(45);
        });

        assert_eq!(cpu.x(), 46);
    }

    #[test]
    fn instr_iny_increments_y_register() {
        let cpu = run_instr(mem!(INY), |cpu| {
            cpu.set_y(45);
        });

        assert_eq!(cpu.y(), 46);
    }

    #[test]
    fn instr_jmp_jumps_to_immediate_operand() {
        let cpu = run_instr(mem!(200 => { JMP_ABSOLUTE, 100, 0 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(200);
        });

        assert_eq!(cpu.program_counter(), Address::new(100));
    }

    #[test]
    fn instr_jmp_jumps_to_indirect_operand() {
        let cpu = run_instr(
            mem!(
                20 => { JMP_INDIRECT, 30, 0 }
                30 => { 10, 0 }
            ),
            |cpu| {
                *cpu.program_counter_mut() = Address::new(20);
            },
        );

        assert_eq!(cpu.program_counter(), Address::new(10));
    }

    #[test]
    fn instr_jsr_jumps_to_operand() {
        let cpu = run_instr(mem!(200 => { JSR, 100, 0 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(200);
        });

        assert_eq!(cpu.program_counter(), Address::new(100));
    }

    #[test]
    fn instr_jsr_writes_program_counter_to_stack_pointer() {
        let cpu = run_instr(mem!(0x1234 => { JSR, 100, 0 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(0x1234);
            cpu.stack_pointer = 6;
        });

        // Program counter points to last byte of JSR instruction
        assert_eq!(cpu.get(STACK + 6), 0x12);
        assert_eq!(cpu.get(STACK + 5), 0x36);
    }

    #[test]
    fn instr_jsr_decrements_stack_pointer_by_two_bytes() {
        let cpu = run_instr(mem!(JSR, 0x23, 0x01), |cpu| {
            cpu.stack_pointer = 6;
        });

        assert_eq!(cpu.stack_pointer, 4);
    }

    #[test]
    fn instr_lda_loads_operand_into_accunmulator() {
        let cpu = run_instr(mem!(LDA_IMMEDIATE, 5u8), |_| {});

        assert_eq!(cpu.accumulator(), 5);
    }

    #[test]
    fn instr_ldx_loads_operand_into_x_register() {
        let cpu = run_instr(mem!(LDX_IMMEDIATE, 5u8), |_| {});

        assert_eq!(cpu.x(), 5);
    }

    #[test]
    fn instr_ldy_loads_operand_into_y_register() {
        let cpu = run_instr(mem!(LDY_IMMEDIATE, 5u8), |_| {});

        assert_eq!(cpu.y(), 5);
    }

    #[test]
    fn instr_lsr_shifts_right() {
        let cpu = run_instr(mem!(LSR_ACCUMULATOR), |cpu| {
            cpu.set_accumulator(0b100);
        });

        assert_eq!(cpu.accumulator(), 0b10);
        assert_eq!(cpu.status.get(Flag::Carry), false);
    }

    #[test]
    fn instr_lsr_sets_carry_flag_on_underflow() {
        let cpu = run_instr(mem!(LSR_ACCUMULATOR), |cpu| {
            cpu.set_accumulator(0b101_0101);
        });

        assert_eq!(cpu.accumulator(), 0b10_1010);
        assert_eq!(cpu.status.get(Flag::Carry), true);
    }

    #[test]
    fn instr_nop_increments_program_counter() {
        let cpu = run_instr(mem!(20 => LSR_ACCUMULATOR), |cpu| {
            *cpu.program_counter_mut() = Address::new(20);
        });

        assert_eq!(cpu.program_counter(), Address::new(21));
    }

    #[test]
    fn instr_ora_performs_bitwise_or() {
        let cpu = run_instr(mem!(ORA_IMMEDIATE, 0b1100_u8), |cpu| {
            cpu.set_accumulator(0b1010);
        });

        assert_eq!(cpu.accumulator(), 0b1110);
    }

    #[test]
    fn instr_pha_writes_accumulator_to_stack_pointer() {
        let cpu = run_instr(mem!(PHA), |cpu| {
            cpu.set_accumulator(20);
            cpu.stack_pointer = 6;
        });

        assert_eq!(cpu.get(STACK + 6), 20);
    }

    #[test]
    fn instr_pha_decrements_stack_pointer_by_one_byte() {
        let cpu = run_instr(mem!(PHA), |cpu| {
            cpu.stack_pointer = 6;
        });

        assert_eq!(cpu.stack_pointer, 5);
    }

    #[test]
    fn instr_php_writes_status_to_stack_pointer() {
        let cpu = run_instr(mem!(PHP), |cpu| {
            cpu.status = Status(142);
            cpu.stack_pointer = 6;
        });

        assert_eq!(cpu.get(STACK + 6), 142);
    }

    #[test]
    fn instr_php_decrements_stack_pointer_by_one_byte() {
        let cpu = run_instr(mem!(PHP), |cpu| {
            cpu.stack_pointer = 6;
        });

        assert_eq!(cpu.stack_pointer, 5);
    }

    #[test]
    fn instr_pla_reads_accumulator_from_stack() {
        let cpu = run_instr(mem!(PLA), |cpu| {
            cpu.set(STACK + 7, 20);
            cpu.stack_pointer = 6;
        });

        assert_eq!(cpu.accumulator(), 20);
    }

    #[test]
    fn instr_pla_increments_stack_pointer_by_one_byte() {
        let cpu = run_instr(mem!(PLA), |cpu| {
            cpu.stack_pointer = 6;
        });

        assert_eq!(cpu.stack_pointer, 7);
    }

    #[test]
    fn instr_plp_reads_status_from_stack() {
        let cpu = run_instr(mem!(PLP), |cpu| {
            cpu.set(STACK, 31);
        });

        assert_eq!(cpu.status.0, 31);
    }

    #[test]
    fn instr_plp_increments_stack_pointer_by_one_byte() {
        let cpu = run_instr(mem!(PLP), |cpu| {
            cpu.stack_pointer = 6;
        });

        assert_eq!(cpu.stack_pointer, 7);
    }

    #[test]
    fn instr_rol_rotates_left_with_carry_flag() {
        let cpu = run_instr(mem!(ROL_ACCUMULATOR), |cpu| {
            cpu.status.clear(Flag::Carry);
            cpu.set_accumulator(0b100);
        });

        assert_eq!(cpu.accumulator(), 0b1000);
        assert_eq!(cpu.status.get(Flag::Carry), false);

        let cpu = run_instr(mem!(ROL_ACCUMULATOR), |cpu| {
            cpu.status.set(Flag::Carry);
            cpu.set_accumulator(0b100);
        });

        assert_eq!(cpu.accumulator(), 0b1001);
        assert_eq!(cpu.status.get(Flag::Carry), false);

        let cpu = run_instr(mem!(ROL_ACCUMULATOR), |cpu| {
            cpu.status.clear(Flag::Carry);
            cpu.set_accumulator(0b1000_0000);
        });

        assert_eq!(cpu.accumulator(), 0);
        assert_eq!(cpu.status.get(Flag::Carry), true);
    }

    #[test]
    fn instr_ror_rotates_left_with_carry_flag() {
        let cpu = run_instr(mem!(ROR_ACCUMULATOR), |cpu| {
            cpu.status.clear(Flag::Carry);
            cpu.set_accumulator(0b100);
        });

        assert_eq!(cpu.accumulator(), 0b10);
        assert_eq!(cpu.status.get(Flag::Carry), false);

        let cpu = run_instr(mem!(ROR_ACCUMULATOR), |cpu| {
            cpu.status.set(Flag::Carry);
            cpu.set_accumulator(0b100);
        });

        assert_eq!(cpu.accumulator(), 0b1000_0010);
        assert_eq!(cpu.status.get(Flag::Carry), false);

        let cpu = run_instr(mem!(ROR_ACCUMULATOR), |cpu| {
            cpu.status.clear(Flag::Carry);
            cpu.set_accumulator(0b1);
        });

        assert_eq!(cpu.accumulator(), 0);
        assert_eq!(cpu.status.get(Flag::Carry), true);
    }

    #[test]
    fn instr_rts_reads_program_counter_plus_one_from_stack() {
        let cpu = run_instr(mem!(RTS), |cpu| {
            cpu.stack_pointer = 100;
            cpu.set(STACK + 102, 0x12);
            cpu.set(STACK + 101, 0x34);
        });

        assert_eq!(cpu.program_counter(), Address::new(0x1235));
    }

    #[test]
    fn instr_rts_increments_stack_pointer_by_two_bytes() {
        let cpu = run_instr(mem!(RTS), |cpu| {
            cpu.stack_pointer = 6;
        });

        assert_eq!(cpu.stack_pointer, 8);
    }

    #[test]
    fn instr_sbc_subtracts_numbers() {
        let cpu = run_instr(mem!(SBC_IMMEDIATE, 10u8), |cpu| {
            cpu.status.set(Flag::Carry);
            cpu.set_accumulator(42);
        });

        assert_eq!(cpu.accumulator(), 32);
        assert_eq!(cpu.status.get(Flag::Carry), true);
    }

    #[test]
    fn instr_sbc_sets_overflow_bit_when_sign_is_wrong() {
        fn sub(accumulator: i8, value: i8) -> (i8, bool) {
            let cpu = run_instr(mem!(SBC_IMMEDIATE, value as u8), |cpu| {
                cpu.status.set(Flag::Carry);
                cpu.set_accumulator(accumulator as u8);
            });

            (cpu.accumulator() as i8, cpu.status.get(Flag::Overflow))
        }

        assert_eq!(sub(80, -16), (96, false));
        assert_eq!(sub(80, -80), (-96, true));
        assert_eq!(sub(80, 112), (-32, false));
        assert_eq!(sub(80, 48), (32, false));
        assert_eq!(sub(-48, -16), (-32, false));
        assert_eq!(sub(-48, -80), (32, false));
        assert_eq!(sub(-48, 112), (96, true));
        assert_eq!(sub(-48, 48), (-96, false));
    }

    #[test]
    fn instr_sec_sets_carry_flag() {
        let cpu = run_instr(mem!(SEC), |cpu| {
            cpu.status.clear(Flag::Carry);
        });

        assert_eq!(cpu.status.get(Flag::Carry), true);
    }

    #[test]
    fn instr_sed_sets_decimal_flag() {
        let cpu = run_instr(mem!(SED), |cpu| {
            cpu.status.clear(Flag::Decimal);
        });

        assert_eq!(cpu.status.get(Flag::Decimal), true);
    }

    #[test]
    fn instr_sei_sets_interrupt_disable_flag() {
        let cpu = run_instr(mem!(SEI), |cpu| {
            cpu.status.clear(Flag::InterruptDisable);
        });

        assert_eq!(cpu.status.get(Flag::InterruptDisable), true);
    }

    #[test]
    fn instr_sta_stores_accumulator_in_memory() {
        let cpu = run_instr(mem!(STA_ABSOLUTE, 0x32, 0), |cpu| {
            cpu.set_accumulator(65);
        });

        assert_eq!(cpu.get(Address::new(0x32)), 65);
    }

    #[test]
    fn instr_stx_stores_x_register_in_memory() {
        let cpu = run_instr(mem!(STX_ABSOLUTE, 0x32, 0), |cpu| {
            cpu.set_x(65);
        });

        assert_eq!(cpu.get(Address::new(0x32)), 65);
    }

    #[test]
    fn instr_sty_stores_y_register_in_memory() {
        let cpu = run_instr(mem!(STY_ABSOLUTE, 0x32, 0), |cpu| {
            cpu.set_y(65);
        });

        assert_eq!(cpu.get(Address::new(0x32)), 65);
    }

    #[test]
    fn instr_tax_transfers_accumulator_to_x_register() {
        let cpu = run_instr(mem!(TAX), |cpu| {
            cpu.set_accumulator(65);
        });

        assert_eq!(cpu.x(), 65);
    }

    #[test]
    fn instr_tay_transfers_accumulator_to_y_register() {
        let cpu = run_instr(mem!(TAY), |cpu| {
            cpu.set_accumulator(65);
        });

        assert_eq!(cpu.y(), 65);
    }

    #[test]
    fn instr_tsx_transfers_stack_pointer_to_x_register() {
        let cpu = run_instr(mem!(TSX), |cpu| {
            cpu.stack_pointer = 65;
        });

        assert_eq!(cpu.x(), 65);
    }

    #[test]
    fn instr_txa_transfers_x_register_to_accumulator() {
        let cpu = run_instr(mem!(TXA), |cpu| {
            cpu.set_x(65);
        });

        assert_eq!(cpu.accumulator(), 65);
    }

    #[test]
    fn instr_txs_transfers_x_register_to_stack_pointer() {
        let cpu = run_instr(mem!(TXS), |cpu| {
            cpu.set_x(65);
        });

        assert_eq!(cpu.stack_pointer, 65);
    }

    #[test]
    fn instr_txs_does_not_modify_zero_or_negative_register() {
        let cpu = run_instr(mem!(TXS), |cpu| {
            cpu.set_x(65);
            cpu.status.set(Flag::Zero);
            cpu.status.set(Flag::Negative);
        });

        assert_eq!(cpu.status.get(Flag::Zero), true);
        assert_eq!(cpu.status.get(Flag::Negative), true);
    }

    #[test]
    fn instr_tya_transfers_y_register_to_accumulator() {
        let cpu = run_instr(mem!(TYA), |cpu| {
            cpu.set_y(65);
        });

        assert_eq!(cpu.accumulator(), 65);
    }

    #[test]
    fn instr_brk_jumps_to_address_at_interrupt_vector() {
        let cpu = run_instr(
            mem!(
                0 => { BRK }
                INTERRUPT_VECTOR => { 0x34, 0x12 }
            ),
            |_| {},
        );

        assert_eq!(cpu.program_counter(), Address::new(0x1234));
    }

    #[test]
    fn instr_brk_writes_program_counter_and_status_to_stack_pointer() {
        let cpu = run_instr(mem!(0x1234 => { BRK }), |cpu| {
            *cpu.program_counter_mut() = Address::new(0x1234);
            cpu.status = Status(0x56);
            cpu.stack_pointer = 6;
        });

        assert_eq!(cpu.get(STACK + 6), 0x12);
        assert_eq!(cpu.get(STACK + 5), 0x34);
        assert_eq!(cpu.get(STACK + 4), 0x56);
    }

    #[test]
    fn instr_brk_decrements_stack_pointer_by_three_bytes() {
        let cpu = run_instr(mem!(BRK), |cpu| {
            cpu.stack_pointer = 6;
        });

        assert_eq!(cpu.stack_pointer, 3);
    }

    #[test]
    fn instr_brk_sets_break_flag_on_stack() {
        let cpu = run_instr(mem!(BRK), |cpu| {
            cpu.status.clear(Flag::Break);
            cpu.stack_pointer = 6;
        });

        let status = Status(cpu.get(STACK + 4));
        assert_eq!(status.get(Flag::Break), true);
    }

    #[test]
    fn instr_rti_reads_status_and_program_counter_from_stack() {
        let cpu = run_instr(mem!(RTI), |cpu| {
            cpu.stack_pointer = 100;
            cpu.set(STACK + 103, 0x12);
            cpu.set(STACK + 102, 0x34);
            cpu.set(STACK + 101, 0x56);
        });

        assert_eq!(cpu.program_counter(), Address::new(0x1234));
        assert_eq!(cpu.status.0, 0x56);
    }

    #[test]
    fn instr_rti_increments_stack_pointer_by_three_bytes() {
        let cpu = run_instr(mem!(RTI), |cpu| {
            cpu.stack_pointer = 6;
        });

        assert_eq!(cpu.stack_pointer, 9);
    }

    #[test]
    fn addition_behaves_appropriately_across_many_values() {
        let carry_values = [true, false];
        let values = [0, 1, 2, 3, 126, 127, 128, 129, 252, 253, 254, 255];

        for x in values.iter() {
            for y in values.iter() {
                for carry_in in carry_values.iter() {
                    let cpu = run_instr(mem!(ADC_IMMEDIATE, *y), |cpu| {
                        cpu.status.set_to(Flag::Carry, *carry_in);
                        cpu.set_accumulator(*x);
                    });

                    let carry_bit = *carry_in as u16;
                    let expected = u16::from(*x) + u16::from(*y) + carry_bit;

                    let carry_out = (cpu.status.get(Flag::Carry) as u16) << 8;
                    let actual = u16::from(cpu.accumulator()) + carry_out;

                    assert_eq!(actual, expected, "{} + {} + {}", x, y, carry_bit);
                }
            }
        }
    }

    #[test]
    fn subtraction_behaves_appropriately_across_many_values() {
        let carry_values = [true, false];
        let values = [0, 1, 2, 3, 126, 127, 128, 129, 252, 253, 254, 255];

        for x in values.iter() {
            for y in values.iter() {
                for carry_in in carry_values.iter() {
                    let cpu = run_instr(mem!(SBC_IMMEDIATE, *y), |cpu| {
                        cpu.status.set_to(Flag::Carry, *carry_in);
                        cpu.set_accumulator(*x);
                    });

                    let carry_bit = *carry_in as u16;
                    let expected = (u16::from(*x))
                        .wrapping_sub(u16::from(*y))
                        .wrapping_sub(1 - carry_bit);
                    let expected = expected & 0b1_1111_1111;

                    let carry_out = cpu.status.get(Flag::Carry) as u16;
                    let accumulator = cpu.accumulator();
                    let actual = u16::from(accumulator) + ((1 - carry_out) << 8);

                    assert_eq!(
                        actual, expected,
                        "\n input: {} - {} - (1 - {})\noutput: {}, carry {} = {}",
                        x, y, carry_bit, accumulator, carry_out, actual
                    );
                }
            }
        }
    }

    #[test]
    fn zero_flag_is_not_set_when_accumulator_is_non_zero() {
        let cpu = run_instr(mem!(ADC_IMMEDIATE, 1u8), |cpu| {
            cpu.set_accumulator(42);
        });

        assert_eq!(cpu.accumulator(), 43);
        assert_eq!(cpu.status.get(Flag::Zero), false);
    }

    #[test]
    fn zero_flag_is_set_when_accumulator_is_zero() {
        let cpu = run_instr(mem!(ADC_IMMEDIATE, 214u8), |cpu| {
            cpu.set_accumulator(42);
        });

        assert_eq!(cpu.accumulator(), 0);
        assert_eq!(cpu.status.get(Flag::Zero), true);
    }

    #[test]
    fn negative_flag_is_not_set_when_accumulator_is_positive() {
        let cpu = run_instr(mem!(ADC_IMMEDIATE, 1u8), |cpu| {
            cpu.set_accumulator(42);
        });

        assert_eq!(cpu.accumulator(), 43);
        assert_eq!(cpu.status.get(Flag::Negative), false);
    }

    #[test]
    fn negative_flag_is_set_when_accumulator_is_negative() {
        let cpu = run_instr(mem!(ADC_IMMEDIATE, -1i8 as u8), |cpu| {
            cpu.set_accumulator(0);
        });

        assert_eq!(cpu.accumulator() as i8, -1i8);
        assert_eq!(cpu.status.get(Flag::Negative), true);
    }

    #[test]
    fn program_counter_is_incremented_by_1_when_executing_1_byte_instr() {
        let cpu = run_instr(mem!(100 => ASL_ACCUMULATOR), |cpu| {
            *cpu.program_counter_mut() = Address::new(100)
        });

        assert_eq!(cpu.program_counter(), Address::new(101));
    }

    #[test]
    fn program_counter_is_incremented_by_2_when_executing_2_byte_instr() {
        let cpu = run_instr(mem!(100 => { ADC_IMMEDIATE, 0u8 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(100)
        });

        assert_eq!(cpu.program_counter(), Address::new(102));
    }

    #[test]
    fn program_counter_is_incremented_by_3_when_executing_3_byte_instr() {
        let cpu = run_instr(mem!(100 => { ASL_ABSOLUTE, 0, 0 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(100)
        });

        assert_eq!(cpu.program_counter(), Address::new(103));
    }

    #[test]
    fn stack_pointer_wraps_on_overflow() {
        let cpu = run_instr(mem!(PLA), |cpu| {
            cpu.stack_pointer = 255;
        });

        assert_eq!(cpu.stack_pointer, 0);

        let cpu = run_instr(mem!(PHA), |cpu| {
            cpu.stack_pointer = 0;
        });

        assert_eq!(cpu.stack_pointer, 255);
    }

    #[test]
    fn stack_operations_wrap_value_on_overflow() {
        let cpu = run_instr(mem!(0x1234 => { JSR, 100, 0 }), |cpu| {
            cpu.stack_pointer = 0;
            *cpu.program_counter_mut() = Address::new(0x1234);
        });

        assert_eq!(cpu.get(STACK), 0x12);
        assert_eq!(cpu.get(STACK + 0xff), 0x36);

        let cpu = run_instr(
            mem!(
                40 => { RTS }
                STACK => { 0x12u8 }
                STACK + 0xff => { 0x36u8 }
            ),
            |cpu| {
                cpu.stack_pointer = 0xfe;
                *cpu.program_counter_mut() = Address::new(40);
            },
        );

        assert_eq!(cpu.program_counter(), Address::new(0x1237));
    }

    #[test]
    fn program_counter_wraps_on_overflow() {
        let cpu = run_instr(mem!(0xffff => NOP), |cpu| {
            *cpu.program_counter_mut() = Address::new(0xffff);
        });

        assert_eq!(cpu.program_counter(), Address::new(0));
    }

    #[test]
    fn instructions_can_wrap_on_program_counter_overflow() {
        let cpu = run_instr(mem!(0xfffe => { JMP_ABSOLUTE, 0x34, 0x12 }), |cpu| {
            *cpu.program_counter_mut() = Address::new(0xfffe);
        });

        assert_eq!(cpu.program_counter(), Address::new(0x1234));
    }

    fn run_instr<F: FnOnce(&mut CPU<ArrayMemory>)>(
        memory: ArrayMemory,
        cpu_setup: F,
    ) -> CPU<ArrayMemory> {
        let mut cpu = CPU::with_memory(memory);

        cpu_setup(&mut cpu);

        cpu.run_instruction();

        hexdump::hexdump(&cpu.memory.slice()[..0x200]);

        cpu
    }

    impl<M: Memory> CPU<M> {
        fn set(&mut self, address: Address, byte: u8) {
            self.write(address, byte);
        }

        fn get(&self, address: Address) -> u8 {
            self.read(address)
        }
    }
}
