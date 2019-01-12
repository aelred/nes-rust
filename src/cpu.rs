use crate::address::Address;
use crate::addressing_modes::ReferenceAddressingMode;
use crate::addressing_modes::ShiftAddressingMode;
use crate::addressing_modes::ValueAddressingMode;
use crate::instructions::Instruction;
use crate::opcodes::OpCode;
use crate::SerializeByte;
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

fn bit0(value: u8) -> bool {
    value & 1 != 0
}

fn bit6(value: u8) -> bool {
    value & (1 << 6) != 0
}

fn bit7(value: u8) -> bool {
    value & (1 << 7) != 0
}

impl CPU {
    pub fn with_memory(memory: &[u8]) -> Self {
        let mut cpu = CPU::default();

        let slice = &mut cpu.addressable.memory[..memory.len()];
        slice.copy_from_slice(&memory);

        cpu
    }

    pub fn read(&self, address: Address) -> u8 {
        self.addressable.deref_address(address)
    }

    pub fn write(&mut self, address: Address, byte: u8) {
        *self.addressable.deref_address_mut(address) = byte;
    }

    pub fn accumulator(&self) -> u8 {
        self.addressable.accumulator
    }

    pub fn run_instruction(&mut self) {
        use crate::instructions::Instruction::*;

        match self.addressable.instr() {
            ADC(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                self.add_to_accumulator(value);
            }
            AND(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                self.set_accumulator(self.accumulator() & value);
            }
            ASL(addressing_mode) => self.shift(addressing_mode, 7, |val, carry| val << 1),
            BCC => self.branch_if(!self.status.get(Flag::Carry)),
            BCS => self.branch_if(self.status.get(Flag::Carry)),
            BEQ => self.branch_if(self.status.get(Flag::Zero)),
            BIT(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                let zero = (self.accumulator() & value) == 0;
                self.status.set_to(Flag::Zero, zero);
                self.status.set_to(Flag::Overflow, bit6(value));
                self.status.set_to(Flag::Negative, bit7(value));
            }
            BMI => self.branch_if(self.status.get(Flag::Negative)),
            BNE => self.branch_if(!self.status.get(Flag::Zero)),
            BPL => self.branch_if(!self.status.get(Flag::Negative)),
            BRK => unimplemented!("BRK"), // TODO
            BVC => self.branch_if(!self.status.get(Flag::Overflow)),
            BVS => self.branch_if(self.status.get(Flag::Overflow)),
            CLC => self.status.clear(Flag::Carry),
            CLD => self.status.clear(Flag::Decimal),
            CLI => self.status.clear(Flag::InterruptDisable),
            CLV => self.status.clear(Flag::Overflow),
            CMP(addressing_mode) => self.compare(self.accumulator(), addressing_mode),
            CPX(addressing_mode) => self.compare(self.x, addressing_mode),
            CPY(addressing_mode) => self.compare(self.y, addressing_mode),
            DEC(addressing_mode) => {
                // Borrow only `addressable` to avoid issue with split borrows
                let addr = self.addressable.fetch_ref(addressing_mode);
                CPU::decrement(&mut self.status, addr);
            }
            DEX => CPU::decrement(&mut self.status, &mut self.x),
            DEY => CPU::decrement(&mut self.status, &mut self.y),
            EOR(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                self.set_accumulator(self.accumulator() ^ value);
            }
            INC(addressing_mode) => {
                // Borrow only `addressable` to avoid issue with split borrows
                let addr = self.addressable.fetch_ref(addressing_mode);
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
                let data = self.program_counter() - 1;
                self.push_stack(data);

                *self.program_counter_mut() = addr;
            }
            LDA(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                self.set_accumulator(value);
            }
            LDX(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                self.x = value;
                self.set_flags(value);
            }
            LDY(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                self.y = value;
                self.set_flags(value);
            }
            LSR(addressing_mode) => self.shift(addressing_mode, 0, |val, carry| val >> 1),
            NOP => {}
            ORA(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                self.set_accumulator(self.accumulator() | value);
            }
            PHA => self.push_stack(self.accumulator()),
            PHP => self.push_stack(self.status),
            PLA => {
                let accumulator = self.pull_stack();
                self.set_accumulator(accumulator);
            }
            PLP => {
                self.status = self.pull_stack();
            }
            ROL(addressing_mode) => {
                self.shift(addressing_mode, 7, |val, carry| (val << 1) | carry);
            }
            ROR(addressing_mode) => {
                self.shift(addressing_mode, 0, |val, carry| val >> 1 | carry << 7);
            }
            RTI => unimplemented!("RTI"), // TODO
            RTS => *self.program_counter_mut() = self.pull_stack(),
            SBC(addressing_mode) => {
                let value = !self.fetch(addressing_mode);
                self.add_to_accumulator(value);
            }
            SEC => self.status.set(Flag::Carry),
            SED => self.status.set(Flag::Decimal),
            SEI => self.status.set(Flag::InterruptDisable),
            STA(addressing_mode) => {
                *self.fetch_ref(addressing_mode) = self.accumulator();
            }
            instr => unimplemented!("{:?}", instr),
        }
    }

    fn add_to_accumulator(&mut self, value: u8) {
        let accumulator = self.accumulator();

        let carry_in = self.status.get(Flag::Carry) as u16;

        let full_result = (accumulator as u16)
            .wrapping_add(value as u16)
            .wrapping_add(carry_in);

        let result = full_result as u8;
        let carry_out = full_result & (1 << 8) != 0;

        // Check if the sign bit has changed
        let overflow = bit7((accumulator ^ result) & (value ^ result));
        self.status.set_to(Flag::Overflow, overflow);

        self.set_accumulator(result);
        self.status.set_to(Flag::Carry, carry_out);
    }

    fn shift(&mut self, mode: ShiftAddressingMode, carry_bit: u8, op: impl FnOnce(u8, u8) -> (u8)) {
        let carry = self.status.get(Flag::Carry);
        let addr = self.fetch_ref(mode);

        let old_value = *addr;
        *addr = op(*addr, carry as u8);
        let carry = old_value & (1 << carry_bit) != 0;
        let new_value = *addr;

        self.status.set_to(Flag::Carry, carry);

        self.set_flags(new_value);
    }

    fn push_stack<T: SerializeBytes>(&mut self, data: T) {
        for byte in data.serialize().rev() {
            let stack_address = STACK + self.stack_pointer;
            let location = self.addressable.deref_address_mut(stack_address);
            *location = byte;
            self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        }
    }

    fn pull_stack<T: SerializeBytes>(&mut self) -> T {
        self.stack_pointer = self.stack_pointer.wrapping_add(T::SIZE);
        let stack_address = STACK + self.stack_pointer;
        self.addressable.deref_address(stack_address)
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

    fn program_counter(&self) -> Address {
        self.addressable.program_counter
    }

    fn program_counter_mut(&mut self) -> &mut Address {
        &mut self.addressable.program_counter
    }

    fn compare<T: ValueAddressingMode>(&mut self, register: u8, addressing_mode: T) {
        let value = self.fetch(addressing_mode);
        let (result, carry) = register.overflowing_sub(value);
        self.status.set_to(Flag::Carry, !carry);
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

    fn fetch_ref<T: ReferenceAddressingMode>(&mut self, addressing_mode: T) -> &mut u8 {
        self.addressable.fetch_ref(addressing_mode)
    }

    pub fn fetch<T: ValueAddressingMode>(&mut self, addressing_mode: T) -> u8 {
        self.addressable.fetch(addressing_mode)
    }
}

impl Default for CPU {
    fn default() -> Self {
        CPU {
            addressable: Addressable {
                memory: [0; 0x10000],
                accumulator: 0,
                program_counter: Address::new(0x00),
            },
            x: 0,
            y: 0,
            stack_pointer: 0xFF,
            status: Status(0),
        }
    }
}

pub struct Addressable {
    /// 2KB of internal RAM, plus more mapped space
    memory: [u8; 0x10000],
    /// A
    accumulator: u8,
    /// PC
    program_counter: Address,
}

impl Addressable {
    pub fn fetch_ref<T: ReferenceAddressingMode>(&mut self, addressing_mode: T) -> &mut u8 {
        addressing_mode.fetch_ref(self)
    }

    pub fn fetch<T: ValueAddressingMode>(&mut self, addressing_mode: T) -> u8 {
        addressing_mode.fetch(self)
    }

    pub fn accumulator(&mut self) -> &mut u8 {
        &mut self.accumulator
    }

    pub fn immediate(&mut self) -> u8 {
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
        self.fetch_at_program_counter()
    }

    pub fn absolute(&mut self) -> &mut u8 {
        let address = self.absolute_address();
        self.deref_address_mut(address)
    }

    pub fn absolute_address(&mut self) -> Address {
        self.fetch_at_program_counter()
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
        let opcode: OpCode = self.fetch_at_program_counter();
        opcode.instruction()
    }

    fn deref_address<T: SerializeBytes>(&self, address: Address) -> T {
        let iter = MemoryIterator::new(&self.memory, address);
        T::deserialize(iter)
    }

    pub fn deref_address_mut(&mut self, address: Address) -> &mut u8 {
        &mut self.memory[address.index()]
    }

    fn fetch_at_program_counter<T: SerializeBytes>(&mut self) -> T {
        let data = self.deref_address(self.program_counter);
        self.program_counter += T::SIZE;
        data
    }
}

struct MemoryIterator<'a> {
    memory: &'a [u8],
    address: Address,
}

impl<'a> MemoryIterator<'a> {
    fn new(memory: &'a [u8], address: Address) -> Self {
        MemoryIterator { memory, address }
    }
}

impl<'a> Iterator for MemoryIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        let byte = self.memory[self.address.index()];
        self.address += 1u8;
        Some(byte)
    }
}

#[derive(Copy, Clone)]
struct Status(u8);

impl Status {
    fn get(&self, flag: Flag) -> bool {
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
        self.set_to(Flag::Negative, bit7(value));
    }
}

impl SerializeByte for Status {
    fn to_byte(&self) -> u8 {
        self.0
    }

    fn from_byte(byte: u8) -> Self {
        Status(byte)
    }
}

enum Flag {
    Negative = 0b10000000,
    Overflow = 0b01000000,
    Decimal = 0b00001000,
    InterruptDisable = 0b00000100,
    Zero = 0b00000010,
    Carry = 0b00000001,
}

#[cfg(test)]
mod tests {
    use super::OpCode::*;
    use super::*;
    use crate::mem;

    #[test]
    fn default_cpu_is_in_default_state() {
        let cpu = CPU::default();

        assert_eq!(cpu.program_counter(), Address::new(0x00));
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
        assert_eq!(cpu.status.get(Flag::Overflow), false);
        assert_eq!(cpu.status.get(Flag::Carry), false);
    }

    #[test]
    fn instr_adc_sets_carry_flag_on_unsigned_overflow() {
        let cpu = run_instr(mem!(ADCImmediate, 255u8), |cpu| {
            *cpu.accumulator_mut() = 42;
        });

        assert_eq!(cpu.accumulator(), 41);
        assert_eq!(cpu.status.get(Flag::Overflow), false);
        assert_eq!(cpu.status.get(Flag::Carry), true);
    }

    #[test]
    fn instr_adc_sets_overflow_flag_on_signed_overflow() {
        let cpu = run_instr(mem!(ADCImmediate, 127i8), |cpu| {
            *cpu.accumulator_mut() = 42i8 as u8;
        });

        assert_eq!(cpu.accumulator() as i8, -87i8);
        assert_eq!(cpu.status.get(Flag::Overflow), true);
        assert_eq!(cpu.status.get(Flag::Carry), false);
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
        assert_eq!(cpu.status.get(Flag::Carry), false);
    }

    #[test]
    fn instr_asl_sets_carry_flag_on_overflow() {
        let cpu = run_instr(mem!(ASLAccumulator), |cpu| {
            *cpu.accumulator_mut() = 0b10101010;
        });

        assert_eq!(cpu.accumulator(), 0b01010100);
        assert_eq!(cpu.status.get(Flag::Carry), true);
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
            cpu.status.clear(Flag::Carry);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_bcc_does_not_branch_when_carry_flag_set() {
        let cpu = run_instr(mem!(BCC, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.set(Flag::Carry);
        });

        assert_eq!(cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_bcs_does_not_branch_when_carry_flag_clear() {
        let cpu = run_instr(mem!(BCS, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.clear(Flag::Carry);
        });

        assert_eq!(cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_bcs_branches_when_carry_flag_set() {
        let cpu = run_instr(mem!(BCS, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.set(Flag::Carry);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_beq_does_not_branch_when_zero_flag_clear() {
        let cpu = run_instr(mem!(BEQ, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.clear(Flag::Zero);
        });

        assert_eq!(cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_beq_branches_when_zero_flag_set() {
        let cpu = run_instr(mem!(BEQ, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.set(Flag::Zero);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_bit_sets_zero_flag_when_bitwise_and_is_zero() {
        let cpu = run_instr(mem!(BITAbsolute, Address::new(654)), |cpu| {
            *cpu.accumulator_mut() = 0b11110000u8;
            cpu.set(Address::new(654), 0b00001111u8);
        });

        assert_eq!(cpu.status.get(Flag::Zero), true);
    }

    #[test]
    fn instr_bit_clears_zero_flag_when_bitwise_and_is_not_zero() {
        let cpu = run_instr(mem!(BITAbsolute, Address::new(654)), |cpu| {
            *cpu.accumulator_mut() = 0b11111100u8;
            cpu.set(Address::new(654), 0b00111111u8);
        });

        assert_eq!(cpu.status.get(Flag::Zero), false);
    }

    #[test]
    fn instr_bit_sets_overflow_bit_based_on_bit_6_of_operand() {
        let cpu = run_instr(mem!(BITAbsolute, Address::new(654)), |cpu| {
            cpu.set(Address::new(654), 0u8);
        });

        assert_eq!(cpu.status.get(Flag::Overflow), false);

        let cpu = run_instr(mem!(BITAbsolute, Address::new(654)), |cpu| {
            cpu.set(Address::new(654), 0b01000000u8);
        });

        assert_eq!(cpu.status.get(Flag::Overflow), true);
    }

    #[test]
    fn instr_bit_sets_negative_bit_based_on_bit_7_of_operand() {
        let cpu = run_instr(mem!(BITAbsolute, Address::new(654)), |cpu| {
            cpu.set(Address::new(654), 0u8);
        });

        assert_eq!(cpu.status.get(Flag::Negative), false);

        let cpu = run_instr(mem!(BITAbsolute, Address::new(654)), |cpu| {
            cpu.set(Address::new(654), 0b10000000u8);
        });

        assert_eq!(cpu.status.get(Flag::Negative), true);
    }

    #[test]
    fn instr_bmi_does_not_branch_when_negative_flag_clear() {
        let cpu = run_instr(mem!(BMI, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.clear(Flag::Negative);
        });

        assert_eq!(cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_bmi_branches_when_negative_flag_set() {
        let cpu = run_instr(mem!(BMI, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.set(Flag::Negative);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_bne_branches_when_zero_flag_clear() {
        let cpu = run_instr(mem!(BNE, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.clear(Flag::Zero);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_bne_does_not_branch_when_zero_flag_set() {
        let cpu = run_instr(mem!(BNE, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.set(Flag::Zero);
        });

        assert_eq!(cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_bpl_branches_when_negative_flag_clear() {
        let cpu = run_instr(mem!(BPL, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.clear(Flag::Negative);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_bpl_does_not_branch_when_negative_flag_set() {
        let cpu = run_instr(mem!(BPL, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.set(Flag::Negative);
        });

        assert_eq!(cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_bvc_branches_when_overflow_flag_clear() {
        let cpu = run_instr(mem!(BVC, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.clear(Flag::Overflow);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter(), Address::new(82));
    }

    #[test]
    fn instr_bvc_does_not_branch_when_overflow_flag_set() {
        let cpu = run_instr(mem!(BVC, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.set(Flag::Overflow);
        });

        assert_eq!(cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_bvs_does_not_branch_when_carry_flag_clear() {
        let cpu = run_instr(mem!(BVS, -10i8), |cpu| {
            *cpu.program_counter_mut() = Address::new(90);
            cpu.status.clear(Flag::Overflow);
        });

        assert_eq!(cpu.program_counter(), Address::new(92));
    }

    #[test]
    fn instr_bvs_branches_when_carry_flag_set() {
        let cpu = run_instr(mem!(BVS, -10i8), |cpu| {
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
        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            *cpu.accumulator_mut() = 1;
        });

        assert_eq!(cpu.status.get(Flag::Carry), false);

        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            *cpu.accumulator_mut() = 10;
        });

        assert_eq!(cpu.status.get(Flag::Carry), true);

        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            *cpu.accumulator_mut() = 100;
        });

        assert_eq!(cpu.status.get(Flag::Carry), true);
    }

    #[test]
    fn instr_cmp_sets_zero_flag_if_accumulator_equals_operand() {
        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            *cpu.accumulator_mut() = 1;
        });

        assert_eq!(cpu.status.get(Flag::Zero), false);

        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            *cpu.accumulator_mut() = 10;
        });

        assert_eq!(cpu.status.get(Flag::Zero), true);

        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            *cpu.accumulator_mut() = 100;
        });

        assert_eq!(cpu.status.get(Flag::Zero), false);
    }

    #[test]
    fn instr_cmp_sets_negative_flag_if_bit_7_of_accumulator_sub_operand_is_set() {
        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            *cpu.accumulator_mut() = 1;
        });

        assert_eq!(cpu.status.get(Flag::Negative), true);

        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            *cpu.accumulator_mut() = 10;
        });

        assert_eq!(cpu.status.get(Flag::Negative), false);

        let cpu = run_instr(mem!(CMPImmediate, 10u8), |cpu| {
            *cpu.accumulator_mut() = 100;
        });

        assert_eq!(cpu.status.get(Flag::Negative), false);
    }

    #[test]
    fn instr_cpx_compares_using_x_register() {
        let cpu = run_instr(mem!(CPXImmediate, 10u8), |cpu| {
            cpu.x = 1;
        });

        assert_eq!(cpu.status.get(Flag::Carry), false);
        assert_eq!(cpu.status.get(Flag::Zero), false);
        assert_eq!(cpu.status.get(Flag::Negative), true);

        let cpu = run_instr(mem!(CPXImmediate, 10u8), |cpu| {
            cpu.x = 10;
        });

        assert_eq!(cpu.status.get(Flag::Carry), true);
        assert_eq!(cpu.status.get(Flag::Zero), true);
        assert_eq!(cpu.status.get(Flag::Negative), false);

        let cpu = run_instr(mem!(CPXImmediate, 10u8), |cpu| {
            cpu.x = 100;
        });

        assert_eq!(cpu.status.get(Flag::Carry), true);
        assert_eq!(cpu.status.get(Flag::Zero), false);
        assert_eq!(cpu.status.get(Flag::Negative), false);
    }

    #[test]
    fn instr_cpy_compares_using_y_register() {
        let cpu = run_instr(mem!(CPYImmediate, 10u8), |cpu| {
            cpu.y = 1;
        });

        assert_eq!(cpu.status.get(Flag::Carry), false);
        assert_eq!(cpu.status.get(Flag::Zero), false);
        assert_eq!(cpu.status.get(Flag::Negative), true);

        let cpu = run_instr(mem!(CPYImmediate, 10u8), |cpu| {
            cpu.y = 10;
        });

        assert_eq!(cpu.status.get(Flag::Carry), true);
        assert_eq!(cpu.status.get(Flag::Zero), true);
        assert_eq!(cpu.status.get(Flag::Negative), false);

        let cpu = run_instr(mem!(CPYImmediate, 10u8), |cpu| {
            cpu.y = 100;
        });

        assert_eq!(cpu.status.get(Flag::Carry), true);
        assert_eq!(cpu.status.get(Flag::Zero), false);
        assert_eq!(cpu.status.get(Flag::Negative), false);
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
        assert_eq!(cpu.status.get(Flag::Zero), false);

        let cpu = run_instr(mem!(DECAbsolute, Address::new(100)), |cpu| {
            cpu.set(Address::new(100), 1);
        });

        assert_eq!(cpu.get(Address::new(100)), 0);
        assert_eq!(cpu.status.get(Flag::Zero), true);
    }

    #[test]
    fn instr_dec_sets_negative_flag_based_on_result() {
        let cpu = run_instr(mem!(DECAbsolute, Address::new(100)), |cpu| {
            cpu.set(Address::new(100), 45);
        });

        assert_eq!(cpu.get(Address::new(100)), 44);
        assert_eq!(cpu.status.get(Flag::Zero), false);

        let cpu = run_instr(mem!(DECAbsolute, Address::new(100)), |cpu| {
            cpu.set(Address::new(100), 0);
        });

        assert_eq!(cpu.get(Address::new(100)) as i8, -1i8);
        assert_eq!(cpu.status.get(Flag::Negative), true);
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
        assert_eq!(cpu.status.get(Flag::Zero), false);

        let cpu = run_instr(mem!(DEX), |cpu| {
            cpu.x = 1;
        });

        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.status.get(Flag::Zero), true);
    }

    #[test]
    fn instr_dex_sets_negative_flag_based_on_result() {
        let cpu = run_instr(mem!(DEX), |cpu| {
            cpu.x = 45;
        });

        assert_eq!(cpu.x, 44);
        assert_eq!(cpu.status.get(Flag::Zero), false);

        let cpu = run_instr(mem!(DEX), |cpu| {
            cpu.x = 0;
        });

        assert_eq!(cpu.x as i8, -1i8);
        assert_eq!(cpu.status.get(Flag::Negative), true);
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
        assert_eq!(cpu.status.get(Flag::Zero), false);

        let cpu = run_instr(mem!(DEY), |cpu| {
            cpu.y = 1;
        });

        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.status.get(Flag::Zero), true);
    }

    #[test]
    fn instr_dey_sets_negative_flag_based_on_result() {
        let cpu = run_instr(mem!(DEY), |cpu| {
            cpu.y = 45;
        });

        assert_eq!(cpu.y, 44);
        assert_eq!(cpu.status.get(Flag::Zero), false);

        let cpu = run_instr(mem!(DEY), |cpu| {
            cpu.y = 0;
        });

        assert_eq!(cpu.y as i8, -1i8);
        assert_eq!(cpu.status.get(Flag::Negative), true);
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
        assert_eq!(cpu.status.get(Flag::Zero), false);

        let cpu = run_instr(mem!(INCAbsolute, Address::new(100)), |cpu| {
            cpu.set(Address::new(100), -1i8 as u8);
        });

        assert_eq!(cpu.get(Address::new(100)), 0);
        assert_eq!(cpu.status.get(Flag::Zero), true);
    }

    #[test]
    fn instr_inc_sets_negative_flag_based_on_result() {
        let cpu = run_instr(mem!(INCAbsolute, Address::new(100)), |cpu| {
            cpu.set(Address::new(100), 45);
        });

        assert_eq!(cpu.get(Address::new(100)), 46);
        assert_eq!(cpu.status.get(Flag::Zero), false);

        let cpu = run_instr(mem!(INCAbsolute, Address::new(100)), |cpu| {
            cpu.set(Address::new(100), -10i8 as u8);
        });

        assert_eq!(cpu.get(Address::new(100)) as i8, -9i8);
        assert_eq!(cpu.status.get(Flag::Negative), true);
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

        assert_eq!(cpu.program_counter(), Address::new(100));
    }

    #[test]
    fn instr_jsr_jumps_to_operand() {
        let cpu = run_instr(mem!(JSR, Address::new(100)), |cpu| {
            *cpu.program_counter_mut() = Address::new(200);
        });

        assert_eq!(cpu.program_counter(), Address::new(100));
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
    fn instr_lsr_shifts_right() {
        let cpu = run_instr(mem!(LSRAccumulator), |cpu| {
            *cpu.accumulator_mut() = 0b100;
        });

        assert_eq!(cpu.accumulator(), 0b10);
        assert_eq!(cpu.status.get(Flag::Carry), false);
    }

    #[test]
    fn instr_lsr_sets_carry_flag_on_underflow() {
        let cpu = run_instr(mem!(LSRAccumulator), |cpu| {
            *cpu.accumulator_mut() = 0b1010101;
        });

        assert_eq!(cpu.accumulator(), 0b101010);
        assert_eq!(cpu.status.get(Flag::Carry), true);
    }

    #[test]
    fn instr_nop_increments_program_counter() {
        let cpu = run_instr(mem!(LSRAccumulator), |cpu| {
            *cpu.program_counter_mut() = Address::new(20);
        });

        assert_eq!(cpu.program_counter(), Address::new(21));
    }

    #[test]
    fn instr_ora_performs_bitwise_or() {
        let cpu = run_instr(mem!(ORAImmediate, 0b1100u8), |cpu| {
            *cpu.accumulator_mut() = 0b1010;
        });

        assert_eq!(cpu.accumulator(), 0b1110);
    }

    #[test]
    fn instr_pha_writes_accumulator_to_stack_pointer() {
        let cpu = run_instr(mem!(PHA), |cpu| {
            *cpu.accumulator_mut() = 20;
            cpu.stack_pointer = 6;
        });

        assert_eq!(cpu.get(STACK + 6u8), 20);
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

        assert_eq!(cpu.get(STACK + 6u8), 142);
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
            cpu.set(STACK + 7u8, 20);
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
        let cpu = run_instr(mem!(ROLAccumulator), |cpu| {
            cpu.status.clear(Flag::Carry);
            *cpu.accumulator_mut() = 0b100;
        });

        assert_eq!(cpu.accumulator(), 0b1000);
        assert_eq!(cpu.status.get(Flag::Carry), false);

        let cpu = run_instr(mem!(ROLAccumulator), |cpu| {
            cpu.status.set(Flag::Carry);
            *cpu.accumulator_mut() = 0b100;
        });

        assert_eq!(cpu.accumulator(), 0b1001);
        assert_eq!(cpu.status.get(Flag::Carry), false);

        let cpu = run_instr(mem!(ROLAccumulator), |cpu| {
            cpu.status.clear(Flag::Carry);
            *cpu.accumulator_mut() = 0b10000000;
        });

        assert_eq!(cpu.accumulator(), 0);
        assert_eq!(cpu.status.get(Flag::Carry), true);
    }

    #[test]
    fn instr_ror_rotates_left_with_carry_flag() {
        let cpu = run_instr(mem!(RORAccumulator), |cpu| {
            cpu.status.clear(Flag::Carry);
            *cpu.accumulator_mut() = 0b100;
        });

        assert_eq!(cpu.accumulator(), 0b10);
        assert_eq!(cpu.status.get(Flag::Carry), false);

        let cpu = run_instr(mem!(RORAccumulator), |cpu| {
            cpu.status.set(Flag::Carry);
            *cpu.accumulator_mut() = 0b100;
        });

        assert_eq!(cpu.accumulator(), 0b10000010);
        assert_eq!(cpu.status.get(Flag::Carry), false);

        let cpu = run_instr(mem!(RORAccumulator), |cpu| {
            cpu.status.clear(Flag::Carry);
            *cpu.accumulator_mut() = 0b1;
        });

        assert_eq!(cpu.accumulator(), 0);
        assert_eq!(cpu.status.get(Flag::Carry), true);
    }

    #[test]
    fn instr_rts_reads_program_counter_from_stack() {
        let cpu = run_instr(mem!(RTS), |cpu| {
            cpu.set(STACK + 2u8, 0x12);
            cpu.set(STACK + 1u8, 0x34);
        });

        assert_eq!(cpu.program_counter(), Address::new(0x1234));
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
        let cpu = run_instr(mem!(SBCImmediate, 10u8), |cpu| {
            cpu.status.set(Flag::Carry);
            *cpu.accumulator_mut() = 42;
        });

        assert_eq!(cpu.accumulator(), 32);
        assert_eq!(cpu.status.get(Flag::Carry), true);
    }

    #[test]
    fn instr_sbc_sets_overflow_bit_when_sign_is_wrong() {
        fn sub(accumulator: i8, value: i8) -> (i8, bool) {
            let cpu = run_instr(mem!(SBCImmediate, value as i8), |cpu| {
                cpu.status.set(Flag::Carry);
                *cpu.accumulator_mut() = accumulator as u8;
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
        let cpu = run_instr(mem!(STAAbsolute, Address::new(0x32)), |cpu| {
            *cpu.accumulator_mut() = 65;
        });

        assert_eq!(cpu.get(Address::new(0x32)), 65);
    }

    #[test]
    fn addition_behaves_appropriately_across_many_values() {
        let carry_values = [true, false];
        let values = [0, 1, 2, 3, 126, 127, 128, 129, 252, 253, 254, 255];

        for x in values.iter() {
            for y in values.iter() {
                for carry_in in carry_values.iter() {
                    let cpu = run_instr(mem!(ADCImmediate, *y), |cpu| {
                        cpu.status.set_to(Flag::Carry, *carry_in);
                        *cpu.accumulator_mut() = *x;
                    });

                    let carry_bit = *carry_in as u16;
                    let expected = *x as u16 + *y as u16 + carry_bit;

                    let carry_out = (cpu.status.get(Flag::Carry) as u16) << 8;
                    let actual = cpu.accumulator() as u16 + carry_out;

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
                    let cpu = run_instr(mem!(SBCImmediate, *y), |cpu| {
                        cpu.status.set_to(Flag::Carry, *carry_in);
                        *cpu.accumulator_mut() = *x;
                    });

                    let carry_bit = *carry_in as u16;
                    let expected = (*x as u16)
                        .wrapping_sub(*y as u16)
                        .wrapping_sub(1 - carry_bit);
                    let expected = expected & 0b1_1111_1111;

                    let carry_out = cpu.status.get(Flag::Carry) as u16;
                    let accumulator = cpu.accumulator();
                    let actual = accumulator as u16 + ((1 - carry_out) << 8);

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
    fn immediate_addressing_mode_fetches_given_value() {
        let mut cpu = CPU::default();
        cpu.set(cpu.program_counter(), 56);
        assert_eq!(cpu.addressable.immediate(), 56);
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
        cpu.set(cpu.program_counter(), lower);
        cpu.set(cpu.program_counter() + 1u16, higher);
        cpu.set(Address::new(432), 35);
        assert_eq!(*cpu.addressable.absolute(), 35);
    }

    #[test]
    fn zero_flag_is_not_set_when_accumulator_is_non_zero() {
        let cpu = run_instr(mem!(ADCImmediate, 1u8), |cpu| {
            *cpu.accumulator_mut() = 42;
        });

        assert_eq!(cpu.accumulator(), 43);
        assert_eq!(cpu.status.get(Flag::Zero), false);
    }

    #[test]
    fn zero_flag_is_set_when_accumulator_is_zero() {
        let cpu = run_instr(mem!(ADCImmediate, 214u8), |cpu| {
            *cpu.accumulator_mut() = 42;
        });

        assert_eq!(cpu.accumulator(), 0);
        assert_eq!(cpu.status.get(Flag::Zero), true);
    }

    #[test]
    fn negative_flag_is_not_set_when_accumulator_is_positive() {
        let cpu = run_instr(mem!(ADCImmediate, 1u8), |cpu| {
            *cpu.accumulator_mut() = 42;
        });

        assert_eq!(cpu.accumulator(), 43);
        assert_eq!(cpu.status.get(Flag::Negative), false);
    }

    #[test]
    fn negative_flag_is_set_when_accumulator_is_negative() {
        let cpu = run_instr(mem!(ADCImmediate, -1i8), |cpu| {
            *cpu.accumulator_mut() = 0;
        });

        assert_eq!(cpu.accumulator() as i8, -1i8);
        assert_eq!(cpu.status.get(Flag::Negative), true);
    }

    #[test]
    fn program_counter_is_incremented_by_1_when_executing_1_byte_instr() {
        let cpu = run_instr(mem!(ASLAccumulator), |cpu| {
            *cpu.program_counter_mut() = Address::new(100)
        });

        assert_eq!(cpu.program_counter(), Address::new(101));
    }

    #[test]
    fn program_counter_is_incremented_by_2_when_executing_2_byte_instr() {
        let cpu = run_instr(mem!(ADCImmediate, 0u8), |cpu| {
            *cpu.program_counter_mut() = Address::new(100)
        });

        assert_eq!(cpu.program_counter(), Address::new(102));
    }

    #[test]
    fn program_counter_is_incremented_by_3_when_executing_3_byte_instr() {
        let cpu = run_instr(mem!(ASLAbsolute, Address::new(0)), |cpu| {
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
        let cpu = run_instr(mem!(JSR, Address::new(100)), |cpu| {
            cpu.stack_pointer = 0;
            *cpu.program_counter_mut() = Address::new(0x1234);
        });

        assert_eq!(cpu.get(STACK), 0x12);
        assert_eq!(cpu.get(STACK + 0xffu8), 0x36);
    }

    #[test]
    fn program_counter_wraps_on_overflow() {
        let cpu = run_instr(mem!(NOP), |cpu| {
            *cpu.program_counter_mut() = Address::new(0xffff);
        });

        assert_eq!(cpu.program_counter(), Address::new(0));
    }

    #[test]
    fn instructions_can_wrap_on_program_counter_overflow() {
        let cpu = run_instr(mem!(JMPAbsolute, Address::new(0x1234)), |cpu| {
            *cpu.program_counter_mut() = Address::new(0xfffe);
        });

        assert_eq!(cpu.program_counter(), Address::new(0x1234));
    }

    fn run_instr<F: FnOnce(&mut CPU)>(data: Vec<u8>, cpu_setup: F) -> CPU {
        let mut cpu = CPU::default();

        cpu_setup(&mut cpu);

        let mut pc = cpu.program_counter();

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
