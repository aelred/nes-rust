use crate::address::Address;
use crate::addressing_modes::ValueAddressingMode;
use crate::instructions::Instruction;
use crate::opcodes::OpCode;
use num_traits::FromPrimitive;
use crate::SerializeBytes;

const STACK: Address = Address::new(0x0100);

pub struct CPU {
    addressable: Addressable,
    /// X
    x: u8,
    /// Y
    y: u8,
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
    pub fn with_memory(memory: Vec<u8>) -> Self {
        let mut cpu = CPU::default();

        let slice = &mut cpu.addressable.memory[..memory.len()];
        slice.copy_from_slice(&memory);

        cpu
    }

    pub fn read(&self, address: Address) -> u8 {
        self.addressable.deref_address(address)
    }

    pub fn accumulator(&self) -> u8 {
        self.addressable.accumulator
    }

    pub fn run_instruction(&mut self) {
        use crate::instructions::Instruction::*;

        match self.addressable.instr() {
            ADC(addressing_mode) => {
                let value = *self.fetch(addressing_mode);

                let (result, carry) = self.accumulator().overflowing_add(value);

                // Perform the operation again, but signed, to check for signed overflow
                self.status.overflow = (self.accumulator() as i8).overflowing_add(value as i8).1;

                self.set_accumulator(result);
                self.status.carry = carry;
            }
            AND(addressing_mode) => {
                let value = *self.fetch(addressing_mode);
                self.set_accumulator(self.accumulator() & value);
            }
            ASL(addressing_mode) => {
                let addr = self.fetch(addressing_mode);

                let old_value = *addr;
                *addr <<= 1;
                let new_value = *addr;

                self.status.carry = bit7(old_value);
                self.set_flags(new_value);
            }
            BCC => self.branch_if(!self.status.carry),
            BCS => self.branch_if(self.status.carry),
            BEQ => self.branch_if(self.status.zero),
            BIT(addressing_mode) => {
                let value = *self.fetch(addressing_mode);
                self.status.zero = (self.accumulator() & value) == 0;
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
            CMP(addressing_mode) => self.compare(self.accumulator(), addressing_mode),
            CPX(addressing_mode) => self.compare(self.x, addressing_mode),
            CPY(addressing_mode) => self.compare(self.y, addressing_mode),
            DEC(addressing_mode) => {
                let addr = self.addressable.fetch(addressing_mode);
                CPU::decrement(&mut self.status, addr);
            }
            DEX => CPU::decrement(&mut self.status, &mut self.x),
            DEY => CPU::decrement(&mut self.status, &mut self.y),
            EOR(addressing_mode) => {
                let value = *self.fetch(addressing_mode);
                self.set_accumulator(self.accumulator() ^ value);
            }
            INC(addressing_mode) => {
                let addr = self.addressable.fetch(addressing_mode);
                CPU::increment(&mut self.status, addr);
            }
            INX => CPU::increment(&mut self.status, &mut self.x),
            INY => CPU::increment(&mut self.status, &mut self.y),
            JMP(addressing_mode) => {
                let addr = addressing_mode.fetch_address(&mut self.addressable);
                *self.program_counter_mut() = addr;
            }
            JSR => {
                let addr = self.addressable.absolute_address();

                // For some reason the spec says the pointer must be to the last byte of the JSR
                // instruction...
                let data = (*self.program_counter() - 1).bytes();

                for byte in data.into_iter() {
                    let addr = self.addressable.deref_address_mut(STACK + self.stack_pointer);
                    *addr = byte;
                    self.stack_pointer -= 1;
                }

                *self.program_counter_mut() = addr;
            }
            LDA(addressing_mode) => {
                let value = *self.fetch(addressing_mode);
                self.set_accumulator(value);
            }
            LDX(addressing_mode) => {
                let value = *self.fetch(addressing_mode);
                self.x = value;
                self.set_flags(value);
            }
            LDY(addressing_mode) => {
                let value = *self.fetch(addressing_mode);
                self.y = value;
                self.set_flags(value);
            }
            instr => unimplemented!("{:?}", instr),
        }
    }

    fn increment(status: &mut Status, addr: &mut u8) {
        let value = addr.wrapping_add(1);
        *addr = value;
        status.set_flags(value);
    }

    fn decrement(status: &mut Status, addr: &mut u8) {
        let value = addr.wrapping_sub(1);
        *addr = value;
        status.set_flags(value);
    }

    fn accumulator_mut(&mut self) -> &mut u8 {
        &mut self.addressable.accumulator
    }

    fn program_counter(&self) -> &Address {
        &self.addressable.program_counter
    }

    fn program_counter_mut(&mut self) -> &mut Address {
        &mut self.addressable.program_counter
    }

    fn compare<T: ValueAddressingMode>(&mut self, register: u8, addressing_mode: T) {
        let value = *self.fetch(addressing_mode);
        let (result, carry) = register.overflowing_sub(value);
        self.status.carry = !carry;
        self.set_flags(result);
    }

    fn set_flags(&mut self, value: u8) {
        self.status.set_flags(value);
    }

    fn set_accumulator(&mut self, value: u8) {
        *self.accumulator_mut() = value;
        self.set_flags(value);
    }

    fn branch_if(&mut self, cond: bool) {
        let offset = self.addressable.relative();
        if cond {
            *self.program_counter_mut() += offset;
        }
    }

    fn fetch<T: ValueAddressingMode>(&mut self, addressing_mode: T) -> &mut u8 {
        self.addressable.fetch(addressing_mode)
    }
}

impl Default for CPU {
    fn default() -> Self {
        CPU {
            addressable: Addressable {
                memory: [0; 0xffff],
                accumulator: 0,
                program_counter: Address::new(0x00),
            },
            x: 0,
            y: 0,
            stack_pointer: 0xFF,
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

pub struct Addressable {
    /// 2KB of internal RAM, plus more mapped space
    memory: [u8; 0xffff],
    /// A
    accumulator: u8,
    /// PC
    program_counter: Address,
}

impl Addressable {
    pub fn fetch<T: ValueAddressingMode>(&mut self, addressing_mode: T) -> &mut u8 {
        addressing_mode.fetch(self)
    }

    pub fn accumulator(&mut self) -> &mut u8 {
        &mut self.accumulator
    }

    pub fn immediate(&mut self) -> &mut u8 {
        self.fetch_at_program_counter()
    }

    pub fn zero_page(&mut self) -> &mut u8 {
        unimplemented!();
    }

    pub fn zero_page_x(&mut self) -> &mut u8 {
        unimplemented!();
    }

    pub fn zero_page_y(&mut self) -> &mut u8 {
        unimplemented!();
    }

    pub fn relative(&mut self) -> i8 {
        *self.fetch_at_program_counter() as i8
    }

    pub fn absolute(&mut self) -> &mut u8 {
        let address = self.absolute_address();
        self.deref_address_mut(address)
    }

    pub fn absolute_address(&mut self) -> Address {
        let higher = *self.fetch_at_program_counter();
        let lower = *self.fetch_at_program_counter();
        Address::from_bytes(higher, lower)
    }

    pub fn absolute_x(&mut self) -> &mut u8 {
        unimplemented!();
    }

    pub fn absolute_y(&mut self) -> &mut u8 {
        unimplemented!();
    }

    pub fn indirect_address(&mut self) -> Address {
        unimplemented!();
    }

    pub fn indirect_indexed(&mut self) -> &mut u8 {
        unimplemented!();
    }

    pub fn indexed_indirect(&mut self) -> &mut u8 {
        unimplemented!();
    }

    fn instr(&mut self) -> Instruction {
        let data = *self.fetch_at_program_counter();
        let opcode = OpCode::from_u8(data).expect("Unrecognised opcode");
        opcode.instruction()
    }

    fn deref_address(&self, address: Address) -> u8 {
        self.memory[address.index()]
    }

    pub fn deref_address_mut(&mut self, address: Address) -> &mut u8 {
        &mut self.memory[address.index()]
    }

    fn fetch_at_program_counter(&mut self) -> &mut u8 {
        let old_program_counter = self.program_counter;
        self.program_counter += 1u16;
        self.deref_address_mut(old_program_counter)
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

impl Status {
    fn set_flags(&mut self, value: u8) {
        self.zero = value == 0;
        self.negative = bit7(value);
    }
}

#[cfg(test)]
mod tests {
    use super::OpCode::*;
    use crate::mem;
    use super::*;

    #[test]
    fn default_cpu_is_in_default_state() {
        let cpu = CPU::default();

        assert_eq!(*cpu.program_counter(), Address::new(0x00));
        assert_eq!(cpu.accumulator(), 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.stack_pointer, 0xFF);
    }

    #[test]
    fn instr_adc_adds_numbers() {
        let cpu = run_instr(mem!(ADCImmediate, 10u8), |cpu| {
            *cpu.accumulator_mut() = 42;
        });

        assert_eq!(cpu.accumulator(), 52);
        assert_eq!(cpu.status.overflow, false);
        assert_eq!(cpu.status.carry, false);
    }

    #[test]
    fn instr_adc_sets_carry_flag_on_unsigned_overflow() {
        let cpu = run_instr(mem!(ADCImmediate, 255u8), |cpu| {
            *cpu.accumulator_mut() = 42;
        });

        assert_eq!(cpu.accumulator(), 41);
        assert_eq!(cpu.status.overflow, false);
        assert_eq!(cpu.status.carry, true);
    }

    #[test]
    fn instr_adc_sets_overflow_flag_on_signed_overflow() {
        let cpu = run_instr(mem!(ADCImmediate, 127i8), |cpu| {
            *cpu.accumulator_mut() = 42i8 as u8;
        });

        assert_eq!(cpu.accumulator() as i8, -87i8);
        assert_eq!(cpu.status.overflow, true);
        assert_eq!(cpu.status.carry, false);
    }

    #[test]
    fn instr_and_performs_bitwise_and() {
        let cpu = run_instr(mem!(ANDImmediate, 0b1100u8), |cpu| {
            *cpu.accumulator_mut() = 0b1010;
        });

        assert_eq!(cpu.accumulator(), 0b1000);
    }

    #[test]
    fn instr_asl_shifts_left() {
        let cpu = run_instr(mem!(ASLAccumulator), |cpu| {
            *cpu.accumulator_mut() = 0b100;
        });

        assert_eq!(cpu.accumulator(), 0b1000);
        assert_eq!(cpu.status.carry, false);
    }

    #[test]
    fn instr_asl_sets_carry_flag_on_overflow() {
        let cpu = run_instr(mem!(ASLAccumulator), |cpu| {
            *cpu.accumulator_mut() = 0b10101010;
        });

        assert_eq!(cpu.accumulator(), 0b01010100);
        assert_eq!(cpu.status.carry, true);
    }

    #[test]
    fn instr_asl_can_operate_on_memory() {
        let cpu = run_instr(mem!(ASLAbsolute, Address::new(100)), |cpu| {
            cpu.set(Address::new(100), 0b100);
        });

        assert_eq!(cpu.get(Address::new(100)), 0b1000);
    }

    #[test]
    fn instr_bcc_branches_when_carry_flag_clear() {
        let cpu = run_instr(mem!(BCC, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.carry = false;
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(*cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_bcc_does_not_branch_when_carry_flag_set() {
        let cpu = run_instr(mem!(BCC, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.carry = true;
        });

        assert_eq!(*cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_bcs_does_not_branch_when_carry_flag_clear() {
        let cpu = run_instr(mem!(BCS, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.carry = false;
        });

        assert_eq!(*cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_bcs_branches_when_carry_flag_set() {
        let cpu = run_instr(mem!(BCS, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.carry = true;
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(*cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_beq_does_not_branch_when_zero_flag_clear() {
        let cpu = run_instr(mem!(BEQ, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.zero = false;
        });

        assert_eq!(*cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_beq_branches_when_zero_flag_set() {
        let cpu = run_instr(mem!(BEQ, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.zero = true;
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(*cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_bit_sets_zero_flag_when_bitwise_and_is_zero() {
        let cpu = run_instr(mem!(BITAbsolute, Address::new(654)), |cpu| {
            *cpu.accumulator_mut() = 0b11110000u8;
            cpu.set(Address::new(654), 0b00001111u8);
        });

        assert_eq!(cpu.status.zero, true);
    }

    #[test]
    fn instr_bit_clears_zero_flag_when_bitwise_and_is_not_zero() {
        let cpu = run_instr(mem!(BITAbsolute, Address::new(654)), |cpu| {
            *cpu.accumulator_mut() = 0b11111100u8;
            cpu.set(Address::new(654), 0b00111111u8);
        });

        assert_eq!(cpu.status.zero, false);
    }

    #[test]
    fn instr_bit_sets_overflow_bit_based_on_bit_6_of_operand() {
        let cpu = run_instr(mem!(BITAbsolute, Address::new(654)), |cpu| {
            cpu.set(Address::new(654), 0u8);
        });

        assert_eq!(cpu.status.overflow, false);

        let cpu = run_instr(mem!(BITAbsolute, Address::new(654)), |cpu| {
            cpu.set(Address::new(654), 0b01000000u8);
        });

        assert_eq!(cpu.status.overflow, true);
    }

    #[test]
    fn instr_bit_sets_negative_bit_based_on_bit_7_of_operand() {
        let cpu = run_instr(mem!(BITAbsolute, Address::new(654)), |cpu| {
            cpu.set(Address::new(654), 0u8);
        });

        assert_eq!(cpu.status.negative, false);

        let cpu = run_instr(mem!(BITAbsolute, Address::new(654)), |cpu| {
            cpu.set(Address::new(654), 0b10000000u8);
        });

        assert_eq!(cpu.status.negative, true);
    }

    #[test]
    fn instr_bmi_does_not_branch_when_negative_flag_clear() {
        let cpu = run_instr(mem!(BMI, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.negative = false;
        });

        assert_eq!(*cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_bmi_branches_when_negative_flag_set() {
        let cpu = run_instr(mem!(BMI, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.negative = true;
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(*cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_bne_branches_when_zero_flag_clear() {
        let cpu = run_instr(mem!(BNE, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.zero = false;
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(*cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_bne_does_not_branch_when_zero_flag_set() {
        let cpu = run_instr(mem!(BNE, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.zero = true;
        });

        assert_eq!(*cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_bpl_branches_when_negative_flag_clear() {
        let cpu = run_instr(mem!(BPL, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.negative = false;
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(*cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_bpl_does_not_branch_when_negative_flag_set() {
        let cpu = run_instr(mem!(BPL, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.negative = true;
        });

        assert_eq!(*cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_bvc_branches_when_overflow_flag_clear() {
        let cpu = run_instr(mem!(BVC, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.overflow = false;
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(*cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_bvc_does_not_branch_when_overflow_flag_set() {
        let cpu = run_instr(mem!(BVC, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.overflow = true;
        });

        assert_eq!(*cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_bvs_does_not_branch_when_carry_flag_clear() {
        let cpu = run_instr(mem!(BVS, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.overflow = false;
        });

        assert_eq!(*cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_bvs_branches_when_carry_flag_set() {
        let cpu = run_instr(mem!(BVS, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.overflow = true;
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(*cpu.program_counter(), Address::new(82));
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
            *cpu.accumulator_mut() = 1;
        });

        assert_eq!(cpu.status.carry, false);

        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            *cpu.accumulator_mut() = 10;
        });

        assert_eq!(cpu.status.carry, true);

        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            *cpu.accumulator_mut() = 100;
        });

        assert_eq!(cpu.status.carry, true);
    }

    #[test]
    fn instr_cmp_sets_zero_flag_if_accumulator_equals_operand() {
        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            *cpu.accumulator_mut() = 1;
        });

        assert_eq!(cpu.status.zero, false);

        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            *cpu.accumulator_mut() = 10;
        });

        assert_eq!(cpu.status.zero, true);

        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            *cpu.accumulator_mut() = 100;
        });

        assert_eq!(cpu.status.zero, false);
    }

    #[test]
    fn instr_cmp_sets_negative_flag_if_bit_7_of_accumulator_sub_operand_is_set() {
        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            *cpu.accumulator_mut() = 1;
        });

        assert_eq!(cpu.status.negative, true);

        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            *cpu.accumulator_mut() = 10;
        });

        assert_eq!(cpu.status.negative, false);

        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            *cpu.accumulator_mut() = 100;
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
        let cpu = run_instr(mem!(DECAbsolute, Address::new(100)), |cpu| {
            cpu.set(Address::new(100), 45);
        });

        assert_eq!(cpu.get(Address::new(100)), 44);
    }

    #[test]
    fn instr_dec_sets_zero_flag_based_on_result() {
        let cpu = run_instr(mem!(DECAbsolute, Address::new(100)), |cpu| {
            cpu.set(Address::new(100), 45);
        });

        assert_eq!(cpu.get(Address::new(100)), 44);
        assert_eq!(cpu.status.zero, false);

        let cpu = run_instr(mem!(DECAbsolute, Address::new(100)), |cpu| {
            cpu.set(Address::new(100), 1);
        });

        assert_eq!(cpu.get(Address::new(100)), 0);
        assert_eq!(cpu.status.zero, true);
    }

    #[test]
    fn instr_dec_sets_negative_flag_based_on_result() {
        let cpu = run_instr(mem!(DECAbsolute, Address::new(100)), |cpu| {
            cpu.set(Address::new(100), 45);
        });

        assert_eq!(cpu.get(Address::new(100)), 44);
        assert_eq!(cpu.status.zero, false);

        let cpu = run_instr(mem!(DECAbsolute, Address::new(100)), |cpu| {
            cpu.set(Address::new(100), 0);
        });

        assert_eq!(cpu.get(Address::new(100)) as i8, -1i8);
        assert_eq!(cpu.status.negative, true);
    }

    #[test]
    fn instr_dex_decrements_x_register() {
        let cpu = run_instr(mem!(DEX), |cpu| {
            cpu.x = 45;
        });

        assert_eq!(cpu.x, 44);
    }

    #[test]
    fn instr_dex_sets_zero_flag_based_on_result() {
        let cpu = run_instr(mem!(DEX), |cpu| {
            cpu.x = 45;
        });

        assert_eq!(cpu.x, 44);
        assert_eq!(cpu.status.zero, false);

        let cpu = run_instr(mem!(DEX), |cpu| {
            cpu.x = 1;
        });

        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.status.zero, true);
    }

    #[test]
    fn instr_dex_sets_negative_flag_based_on_result() {
        let cpu = run_instr(mem!(DEX), |cpu| {
            cpu.x = 45;
        });

        assert_eq!(cpu.x, 44);
        assert_eq!(cpu.status.zero, false);

        let cpu = run_instr(mem!(DEX), |cpu| {
            cpu.x = 0;
        });

        assert_eq!(cpu.x as i8, -1i8);
        assert_eq!(cpu.status.negative, true);
    }

    #[test]
    fn instr_dey_decrements_y_register() {
        let cpu = run_instr(mem!(DEY), |cpu| {
            cpu.y = 45;
        });

        assert_eq!(cpu.y, 44);
    }

    #[test]
    fn instr_dey_sets_zero_flag_based_on_result() {
        let cpu = run_instr(mem!(DEY), |cpu| {
            cpu.y = 45;
        });

        assert_eq!(cpu.y, 44);
        assert_eq!(cpu.status.zero, false);

        let cpu = run_instr(mem!(DEY), |cpu| {
            cpu.y = 1;
        });

        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.status.zero, true);
    }

    #[test]
    fn instr_dey_sets_negative_flag_based_on_result() {
        let cpu = run_instr(mem!(DEY), |cpu| {
            cpu.y = 45;
        });

        assert_eq!(cpu.y, 44);
        assert_eq!(cpu.status.zero, false);

        let cpu = run_instr(mem!(DEY), |cpu| {
            cpu.y = 0;
        });

        assert_eq!(cpu.y as i8, -1i8);
        assert_eq!(cpu.status.negative, true);
    }

    #[test]
    fn instr_eor_performs_bitwise_xor() {
        let cpu = run_instr(mem!(EORImmediate, 0b1100u8), |cpu| {
            *cpu.accumulator_mut() = 0b1010;
        });

        assert_eq!(cpu.accumulator(), 0b0110);
    }

    #[test]
    fn instr_inc_increments_operand() {
        let cpu = run_instr(mem!(INCAbsolute, Address::new(100)), |cpu| {
            cpu.set(Address::new(100), 45);
        });

        assert_eq!(cpu.get(Address::new(100)), 46);
    }

    #[test]
    fn instr_inc_sets_zero_flag_based_on_result() {
        let cpu = run_instr(mem!(INCAbsolute, Address::new(100)), |cpu| {
            cpu.set(Address::new(100), 45);
        });

        assert_eq!(cpu.get(Address::new(100)), 46);
        assert_eq!(cpu.status.zero, false);

        let cpu = run_instr(mem!(INCAbsolute, Address::new(100)), |cpu| {
            cpu.set(Address::new(100), -1i8 as u8);
        });

        assert_eq!(cpu.get(Address::new(100)), 0);
        assert_eq!(cpu.status.zero, true);
    }

    #[test]
    fn instr_inc_sets_negative_flag_based_on_result() {
        let cpu = run_instr(mem!(INCAbsolute, Address::new(100)), |cpu| {
            cpu.set(Address::new(100), 45);
        });

        assert_eq!(cpu.get(Address::new(100)), 46);
        assert_eq!(cpu.status.zero, false);

        let cpu = run_instr(mem!(INCAbsolute, Address::new(100)), |cpu| {
            cpu.set(Address::new(100), -10i8 as u8);
        });

        assert_eq!(cpu.get(Address::new(100)) as i8, -9i8);
        assert_eq!(cpu.status.negative, true);
    }

    #[test]
    fn instr_inx_increments_x_register() {
        let cpu = run_instr(mem!(INX), |cpu| {
            cpu.x = 45;
        });

        assert_eq!(cpu.x, 46);
    }

    #[test]
    fn instr_iny_increments_y_register() {
        let cpu = run_instr(mem!(INY), |cpu| {
            cpu.y = 45;
        });

        assert_eq!(cpu.y, 46);
    }

    #[test]
    fn instr_jmp_jumps_to_operand() {
        let cpu = run_instr(mem!(JMPAbsolute, Address::new(100)), |cpu| {
            *cpu.program_counter_mut() = Address::new(200);
        });

        assert_eq!(*cpu.program_counter(), Address::new(100));
    }

    #[test]
    fn instr_jsr_jumps_to_operand() {
        let cpu = run_instr(mem!(JSR, Address::new(100)), |cpu| {
            *cpu.program_counter_mut() = Address::new(200);
        });

        assert_eq!(*cpu.program_counter(), Address::new(100));
    }

    #[test]
    fn instr_jsr_writes_program_counter_to_stack_pointer() {
        let cpu = run_instr(mem!(JSR, Address::new(100)), |cpu| {
            *cpu.program_counter_mut() = Address::new(0x1234);
            cpu.stack_pointer = 6;
        });

        // Program counter points to last byte of JSR instruction
        assert_eq!(cpu.get(STACK + 6u8), 0x12);
        assert_eq!(cpu.get(STACK + 5u8), 0x36);
    }

    #[test]
    fn instr_jsr_decrements_stack_pointer_by_two_bytes() {
        let cpu = run_instr(mem!(JSR, Address::new(0x0123)), |cpu| {
            cpu.stack_pointer = 6;
        });

        assert_eq!(cpu.stack_pointer, 4);
    }

    #[test]
    fn instr_lda_loads_operand_into_accumulator() {
        let cpu = run_instr(mem!(LDAImmediate, 5u8), |cpu| {});

        assert_eq!(cpu.accumulator(), 5);
    }

    #[test]
    fn instr_ldx_loads_operand_into_x_register() {
        let cpu = run_instr(mem!(LDXImmediate, 5u8), |cpu| {});

        assert_eq!(cpu.x, 5);
    }

    #[test]
    fn instr_ldy_loads_operand_into_y_register() {
        let cpu = run_instr(mem!(LDYImmediate, 5u8), |cpu| {});

        assert_eq!(cpu.y, 5);
    }

    #[test]
    fn immediate_addressing_mode_fetches_given_value() {
        let mut cpu = CPU::default();
        cpu.set(*cpu.program_counter(), 56);
        assert_eq!(*cpu.addressable.immediate(), 56);
    }

    #[test]
    fn accumulator_addressing_mode_fetches_accumulator_value() {
        let mut cpu = CPU::default();
        *cpu.accumulator_mut() = 76;
        assert_eq!(*cpu.addressable.accumulator(), 76);
    }

    #[test]
    fn absolute_addressing_mode_fetches_values_at_given_address() {
        let mut cpu = CPU::default();
        let (higher, lower) = Address::new(432).split();
        cpu.set(*cpu.program_counter(), higher);
        cpu.set(*cpu.program_counter() + 1u16, lower);
        cpu.set(Address::new(432), 35);
        assert_eq!(*cpu.addressable.absolute(), 35);
    }

    #[test]
    fn zero_flag_is_not_set_when_accumulator_is_non_zero() {
        let cpu = run_instr(mem!(ADCImmediate, 1u8), |cpu| {
            *cpu.accumulator_mut() = 42;
        });

        assert_eq!(cpu.accumulator(), 43);
        assert_eq!(cpu.status.zero, false);
    }

    #[test]
    fn zero_flag_is_set_when_accumulator_is_zero() {
        let cpu = run_instr(mem!(ADCImmediate, 214u8), |cpu| {
            *cpu.accumulator_mut() = 42;
        });

        assert_eq!(cpu.accumulator(), 0);
        assert_eq!(cpu.status.zero, true);
    }

    #[test]
    fn negative_flag_is_not_set_when_accumulator_is_positive() {
        let cpu = run_instr(mem!(ADCImmediate, 1u8), |cpu| {
            *cpu.accumulator_mut() = 42;
        });

        assert_eq!(cpu.accumulator(), 43);
        assert_eq!(cpu.status.negative, false);
    }

    #[test]
    fn negative_flag_is_set_when_accumulator_is_negative() {
        let cpu = run_instr(mem!(ADCImmediate, -1i8), |cpu| {
            *cpu.accumulator_mut() = 0;
        });

        assert_eq!(cpu.accumulator() as i8, -1i8);
        assert_eq!(cpu.status.negative, true);
    }

    #[test]
    fn program_counter_is_incremented_by_1_when_executing_1_byte_instr() {
        let cpu = run_instr(mem!(ASLAccumulator), |cpu| {
            *cpu.program_counter_mut() = Address::new(100)
        });

        assert_eq!(*cpu.program_counter(), Address::new(101));
    }

    #[test]
    fn program_counter_is_incremented_by_2_when_executing_2_byte_instr() {
        let cpu = run_instr(mem!(ADCImmediate, 0u8), |cpu| {
            *cpu.program_counter_mut() = Address::new(100)
        });

        assert_eq!(*cpu.program_counter(), Address::new(102));
    }

    #[test]
    fn program_counter_is_incremented_by_3_when_executing_3_byte_instr() {
        let cpu = run_instr(mem!(ASLAbsolute, Address::new(0)), |cpu| {
            *cpu.program_counter_mut() = Address::new(100)
        });

        assert_eq!(*cpu.program_counter(), Address::new(103));
    }

    fn run_instr<F: FnOnce(&mut CPU)>(data: Vec<u8>, cpu_setup: F) -> CPU {
        let mut cpu = CPU::default();

        cpu_setup(&mut cpu);

        let mut pc = *cpu.program_counter();

        for byte in data.iter() {
            cpu.set(pc, *byte);
            pc += 1u16;
        }

        cpu.run_instruction();

        hexdump::hexdump(&cpu.addressable.memory[..0x200]);

        cpu
    }

    impl CPU {
        fn set(&mut self, address: Address, byte: u8) {
            self.addressable.memory[address.index()] = byte;
        }

        fn get(&self, address: Address) -> u8 {
            self.addressable.memory[address.index()]
        }
    }
}
