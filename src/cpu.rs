//! Emulates a 6502 (the NES CPU).
//!
//! The 6502 has:
//! - 6 registers (A, PC, X, Y, S and P)
//! - A 16-bit address bus
//!
//! An instruction comprises:
//! - A 1-byte opcode, comprising instruction and addressing mode.
//! - 0-2 byte operands.

use std::fmt;
use std::fmt::Debug;

use bitflags::bitflags;
use log::trace;
use stack::StackPointer;

use crate::address::Address;
use crate::memory::Memory;

pub use self::instruction::instructions;
pub use self::instruction::Instruction;
pub use self::memory::NESCPUMemory;

mod addressing_modes;
mod instruction;
mod memory;
mod stack;

const NMI_VECTOR: Address = Address::new(0xFFFA);
const RESET_VECTOR: Address = Address::new(0xFFFC);
const INTERRUPT_VECTOR: Address = Address::new(0xFFFE);

#[derive(Debug)]
pub struct CPU<M = NESCPUMemory> {
    memory: M,
    /// A - 8-bit accumulator register.
    accumulator: u8,
    /// PC - 16-bit program counter register.
    program_counter: Address,
    /// X - 8-bit index register.
    x: u8,
    /// Y - 8-bit index register.
    y: u8,
    /// S - 8-bit stack pointer.
    stack_pointer: StackPointer,
    /// P - 7-bit status register.
    status: Status,
    non_maskable_interrupt: bool,
    // Counts cycles taken running the current instruction.
    cycle_count: u8,
}

impl<M: Memory> CPU<M> {
    pub fn from_memory(mut memory: M) -> Self {
        let lower = memory.read(RESET_VECTOR);
        let higher = memory.read(RESET_VECTOR + 1);
        let program_counter = Address::from_bytes(higher, lower);

        CPU {
            memory,
            accumulator: 0,
            program_counter,
            x: 0,
            y: 0,
            stack_pointer: StackPointer::default(),
            status: Status::empty(),
            non_maskable_interrupt: false,
            cycle_count: 0,
        }
    }

    pub fn program_counter(&self) -> Address {
        self.program_counter
    }

    pub fn set_program_counter(&mut self, address: Address) {
        self.program_counter = address;
    }

    pub fn non_maskable_interrupt(&mut self) {
        self.non_maskable_interrupt = true;
    }

    pub fn memory(&mut self) -> &mut M {
        &mut self.memory
    }

    pub fn read(&mut self, address: Address) -> u8 {
        self.cycle_count += 1;
        self.memory.read(address)
    }

    fn read_address(&mut self, address: Address) -> Address {
        let lower = self.read(address);
        let higher = self.read(address.incr_lower());
        Address::from_bytes(higher, lower)
    }

    pub fn write(&mut self, address: Address, byte: u8) {
        self.cycle_count += 1;
        self.memory.write(address, byte);
    }

    pub fn run_instruction(&mut self) -> u8 {
        self.cycle_count = 0;

        let instruction = Instruction::from_opcode(self.incr_program_counter());
        trace!("        {:?}", instruction);

        if self.non_maskable_interrupt {
            self.non_maskable_interrupt = false;
            self.interrupt(NMI_VECTOR, false);
        } else {
            self.handle_instruction(instruction);
        }

        self.cycle_count
    }

    fn handle_instruction(&mut self, instruction: Instruction) {
        use self::instruction::Instruction::*;

        match instruction {
            // Load/Store Operations
            LDA(addressing_mode) => self.lda(addressing_mode),
            LDX(addressing_mode) => self.ldx(addressing_mode),
            LDY(addressing_mode) => self.ldy(addressing_mode),
            STA(addressing_mode) => self.sta(addressing_mode),
            STX(addressing_mode) => self.stx(addressing_mode),
            STY(addressing_mode) => self.sty(addressing_mode),

            // Register Transfers
            TAX => self.tax(),
            TAY => self.tay(),
            TXA => self.txa(),
            TYA => self.tya(),
            TSX => self.tsx(),
            TXS => self.txs(),

            // Stack Operations
            PLA => self.pla(),
            PLP => self.plp(),
            PHA => self.pha(),
            PHP => self.php(),

            // Logical
            AND(addressing_mode) => self.and(addressing_mode),
            EOR(addressing_mode) => self.eor(addressing_mode),
            ORA(addressing_mode) => self.ora(addressing_mode),
            BIT(addressing_mode) => self.bit(addressing_mode),

            // Arithmetic
            ADC(addressing_mode) => self.adc(addressing_mode),
            SBC(addressing_mode) => self.sbc(addressing_mode),
            CMP(addressing_mode) => self.cmp(addressing_mode),
            CPX(addressing_mode) => self.cpx(addressing_mode),
            CPY(addressing_mode) => self.cpy(addressing_mode),

            // Increments & Decrements
            INC(addressing_mode) => self.inc(addressing_mode),
            INX => self.inx(),
            INY => self.iny(),
            DEC(addressing_mode) => self.dec(addressing_mode),
            DEX => self.dex(),
            DEY => self.dey(),

            // Shifts
            ASL(addressing_mode) => drop(self.asl(addressing_mode)),
            LSR(addressing_mode) => drop(self.lsr(addressing_mode)),
            ROL(addressing_mode) => drop(self.rol(addressing_mode)),
            ROR(addressing_mode) => drop(self.ror(addressing_mode)),

            // Jumps & Calls
            JMP(addressing_mode) => self.jmp(addressing_mode),
            JSR => self.jsr(),
            RTS => self.rts(),

            // Branches
            BCC => self.bcc(),
            BCS => self.bcs(),
            BEQ => self.beq(),
            BMI => self.bmi(),
            BNE => self.bne(),
            BPL => self.bpl(),
            BVC => self.bvc(),
            BVS => self.bvs(),

            // Status Flag Changes
            CLC => {
                self.ignore_argument();
                self.status.remove(Status::CARRY)
            }
            CLD => {
                self.ignore_argument();
                self.status.remove(Status::DECIMAL)
            }
            CLI => {
                self.ignore_argument();
                self.status.remove(Status::INTERRUPT_DISABLE)
            }
            CLV => {
                self.ignore_argument();
                self.status.remove(Status::OVERFLOW)
            }
            SEC => {
                self.ignore_argument();
                self.status.insert(Status::CARRY)
            }
            SED => {
                self.ignore_argument();
                self.status.insert(Status::DECIMAL)
            }
            SEI => {
                self.ignore_argument();
                self.status.insert(Status::INTERRUPT_DISABLE)
            }

            // System Functions
            BRK => {
                self.ignore_argument();
                self.interrupt(INTERRUPT_VECTOR, true)
            }
            NOP => {
                self.ignore_argument();
            }
            RTI => {
                self.ignore_argument();
                self.increment_stack();
                self.status = Status::from_bits_truncate(self.pull_and_increment_stack());
                let lower = self.pull_and_increment_stack();
                let higher = self.pull_stack();
                self.program_counter = Address::from_bytes(higher, lower);
            }

            // Unofficial Opcodes
            IGN(addressing_mode) => {
                self.fetch_ref(addressing_mode);
            }
            SKB => {
                self.incr_program_counter();
            }
            LAX(addressing_mode) => {
                let value = self.fetch(addressing_mode);
                self.set_accumulator(value);
                self.set_x(value);
            }
            SAX(addressing_mode) => {
                let reference = self.fetch_ref(addressing_mode);
                self.write_reference(reference, self.accumulator & self.x, true);
            }
            DCP(addressing_mode) => {
                let reference = self.fetch_ref(addressing_mode);
                self.decrement(reference);
                let value = self.read_reference(reference, false);
                self.compare(self.accumulator, value);
            }
            ISC(addressing_mode) => {
                let reference = self.fetch_ref(addressing_mode);
                self.increment(reference);
                let value = self.read_reference(reference, false);
                self.sub_from_accumulator(value);
            }
            SLO(addressing_mode) => {
                let value = self.asl(addressing_mode);
                self.set_accumulator(self.accumulator | value);
            }
            RLA(addressing_mode) => {
                let value = self.rol(addressing_mode);
                self.set_accumulator(self.accumulator & value);
            }
            SRE(addressing_mode) => {
                let value = self.lsr(addressing_mode);
                self.set_accumulator(self.accumulator ^ value);
            }
            RRA(addressing_mode) => {
                let value = self.ror(addressing_mode);
                self.add_to_accumulator(value);
            }
        }
    }

    fn sub_from_accumulator(&mut self, value: u8) {
        self.add_to_accumulator(!value);
    }

    fn interrupt(&mut self, address_vector: Address, break_flag: bool) {
        // For some reason the spec says the pointer must be to the last byte of the BRK
        // instruction...
        let data = self.program_counter - 1;

        self.push_stack(data.higher());
        self.push_stack(data.lower());
        self.push_status(break_flag);

        self.program_counter = self.read_address(address_vector);
    }

    fn push_status(&mut self, break_flag: bool) {
        self.status.insert(Status::UNUSED);
        self.status.set(Status::BREAK, break_flag);
        self.push_stack(self.status.bits());
    }

    fn add_to_accumulator(&mut self, value: u8) {
        let accumulator = self.accumulator;

        let carry_in = self.status.contains(Status::CARRY) as u16;

        let full_result = u16::from(accumulator)
            .wrapping_add(u16::from(value))
            .wrapping_add(carry_in);

        let result = full_result as u8;
        let carry_out = full_result & (1 << 8) != 0;

        // Check if the sign bit has changed
        let overflow = (((accumulator ^ result) & (value ^ result)) as i8).is_negative();
        self.status.set(Status::OVERFLOW, overflow);

        self.set_accumulator(result);
        self.status.set(Status::CARRY, carry_out);
    }

    fn increment(&mut self, reference: Reference) {
        let value = self.read_reference(reference, false);
        self.set_reference(reference, value, false); // redundant write
        self.set_reference(reference, value.wrapping_add(1), false);
    }

    fn decrement(&mut self, reference: Reference) {
        let value = self.read_reference(reference, false);
        self.set_reference(reference, value, false); // redundant write
        self.set_reference(reference, value.wrapping_sub(1), false);
    }

    fn compare(&mut self, register: u8, value: u8) {
        let (result, carry) = register.overflowing_sub(value);
        self.status.set(Status::CARRY, !carry);
        self.status.set_flags(result);
    }

    fn set_reference(&mut self, reference: Reference, value: u8, writeonly: bool) {
        self.write_reference(reference, value, writeonly);
        self.status.set_flags(value);
    }

    fn set_accumulator(&mut self, value: u8) {
        self.set_reference(Reference::Accumulator, value, true);
    }

    fn set_x(&mut self, value: u8) {
        self.set_reference(Reference::X, value, true);
    }

    fn set_y(&mut self, value: u8) {
        self.set_reference(Reference::Y, value, true);
    }

    fn fetch_ref<T: ReferenceAddressingMode>(&mut self, addressing_mode: T) -> Reference {
        addressing_mode.fetch_ref(self)
    }

    fn fetch<T: ReferenceAddressingMode>(&mut self, addressing_mode: T) -> u8 {
        let reference = self.fetch_ref(addressing_mode);
        self.read_reference(reference, true)
    }

    fn read_reference(&mut self, reference: Reference, readonly: bool) -> u8 {
        match reference {
            Reference::Immediate(value) => value,
            Reference::Address(address) => self.read(address),
            Reference::IndexedAddress {
                address,
                page_cross,
            } => {
                if page_cross || !readonly {
                    self.cycle_count += 1;
                }
                self.read(address)
            }
            Reference::Accumulator => self.accumulator,
            Reference::X => self.x,
            Reference::Y => self.y,
        }
    }

    fn write_reference(&mut self, reference: Reference, byte: u8, writeonly: bool) {
        trace!("        {} := {:<#04x}", reference, byte);
        match reference {
            Reference::Immediate(_) => panic!("Tried to write to immediate reference"),
            Reference::Address(address) => {
                self.write(address, byte);
            }
            Reference::IndexedAddress {
                address,
                page_cross: _,
            } => {
                // Redundant read
                if writeonly {
                    self.cycle_count += 1;
                }
                self.write(address, byte)
            }
            Reference::Accumulator => self.accumulator = byte,
            Reference::X => self.x = byte,
            Reference::Y => self.y = byte,
        };
    }

    fn incr_program_counter(&mut self) -> u8 {
        let data = self.read(self.program_counter);
        trace!("{}  {:#04x}", self.program_counter, data);
        self.program_counter += 1u16;
        data
    }

    /// When instructions have no arguments the CPU still reads the value - emulate to make the clock cycle is correct.
    fn ignore_argument(&mut self) {
        self.read(self.program_counter);
    }

    fn fetch_address_at_program_counter(&mut self) -> Address {
        let lower = self.incr_program_counter();
        let higher = self.incr_program_counter();
        Address::from_bytes(higher, lower)
    }
}

trait ReferenceAddressingMode {
    fn fetch_ref<M: Memory>(self, cpu: &mut CPU<M>) -> Reference;
}

#[derive(Copy, Clone)]
enum Reference {
    Immediate(u8),
    Address(Address),
    // Some addressing modes will re-read the value (which impacts cycle count)
    IndexedAddress { address: Address, page_cross: bool },
    Accumulator,
    X,
    Y,
}

impl Reference {
    pub fn indexed_address(base: Address, offset: u8) -> Reference {
        let address = base + u16::from(offset);
        Reference::IndexedAddress {
            address,
            page_cross: address.page_crossed(base),
        }
    }
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Reference::Immediate(value) => write!(f, "#{}", value),
            Reference::Address(address) => write!(f, "{}", address),
            Reference::IndexedAddress {
                address,
                page_cross: _,
            } => write!(f, "{} (x2)", address),
            Reference::Accumulator => f.write_str("A"),
            Reference::X => f.write_str("X"),
            Reference::Y => f.write_str("Y"),
        }
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug)]
    struct Status: u8 {
        const NEGATIVE          = 0b1000_0000;
        const OVERFLOW          = 0b0100_0000;
        const UNUSED            = 0b0010_0000;
        const BREAK             = 0b0001_0000;
        const DECIMAL           = 0b0000_1000;
        const INTERRUPT_DISABLE = 0b0000_0100;
        const ZERO              = 0b0000_0010;
        const CARRY             = 0b0000_0001;
    }
}

impl Status {
    fn set_flags(&mut self, value: u8) {
        self.set(Status::ZERO, value == 0);
        self.set(Status::NEGATIVE, (value as i8).is_negative());
    }
}

#[cfg(test)]
mod tests {
    use yare::parameterized;

    use crate::cpu::addressing_modes::{
        BITAddressingMode, CompareAddressingMode, FlexibleAddressingMode, IncDecAddressingMode,
        JumpAddressingMode, LDXAddressingMode, LDYAddressingMode, STXAddressingMode,
        STYAddressingMode, ShiftAddressingMode, StoreAddressingMode,
    };
    use crate::mem;
    use crate::ArrayMemory;
    use crate::Instruction::{
        ADC, AND, ASL, BCC, BIT, CMP, CPX, CPY, DEC, EOR, INC, JMP, LDA, LDX, LDY, LSR, ORA, ROL,
        ROR, SBC, STA, STX, STY, TAX,
    };

    use super::instructions::*;
    use super::*;

    #[test]
    fn cpu_initialises_in_default_state() {
        let mut memory = ArrayMemory::default();
        let cpu = CPU::from_memory(&mut memory);

        assert_eq!(cpu.program_counter, Address::new(0x00));
        assert_eq!(cpu.accumulator, 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.stack_pointer, StackPointer(0xFF));
    }

    #[test]
    fn cpu_initialises_program_counter_to_reset_vector() {
        let mut memory = mem! {
            0xFFFC => { 0x34, 0x12 }
        };

        let cpu = CPU::from_memory(&mut memory);

        assert_eq!(cpu.program_counter, Address::new(0x1234));
    }

    #[test]
    fn instr_clc_clears_carry_flag() {
        let cpu = run_instr(mem!(CLC), |cpu| {
            cpu.status.insert(Status::CARRY);
        });

        assert!(!cpu.status.contains(Status::CARRY));
    }

    #[test]
    fn instr_cld_clears_decimal_flag() {
        let cpu = run_instr(mem!(CLD), |cpu| {
            cpu.status.insert(Status::DECIMAL);
        });

        assert!(!cpu.status.contains(Status::DECIMAL));
    }

    #[test]
    fn instr_cli_clears_interrupt_disable_flag() {
        let cpu = run_instr(mem!(CLI), |cpu| {
            cpu.status.insert(Status::INTERRUPT_DISABLE);
        });

        assert!(!cpu.status.contains(Status::INTERRUPT_DISABLE));
    }

    #[test]
    fn instr_clv_clears_overflow_flag() {
        let cpu = run_instr(mem!(CLV), |cpu| {
            cpu.status.insert(Status::OVERFLOW);
        });

        assert!(!cpu.status.contains(Status::OVERFLOW));
    }

    #[test]
    fn instr_nop_increments_program_counter() {
        let cpu = run_instr(mem!(20 => LSR_ACCUMULATOR), |cpu| {
            cpu.program_counter = Address::new(20);
        });

        assert_eq!(cpu.program_counter, Address::new(21));
    }

    #[test]
    fn instr_sec_sets_carry_flag() {
        let cpu = run_instr(mem!(SEC), |cpu| {
            cpu.status.remove(Status::CARRY);
        });

        assert!(cpu.status.contains(Status::CARRY));
    }

    #[test]
    fn instr_sed_sets_decimal_flag() {
        let cpu = run_instr(mem!(SED), |cpu| {
            cpu.status.remove(Status::DECIMAL);
        });

        assert!(cpu.status.contains(Status::DECIMAL));
    }

    #[test]
    fn instr_sei_sets_interrupt_disable_flag() {
        let cpu = run_instr(mem!(SEI), |cpu| {
            cpu.status.remove(Status::INTERRUPT_DISABLE);
        });

        assert!(cpu.status.contains(Status::INTERRUPT_DISABLE));
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

        assert_eq!(cpu.program_counter, Address::new(0x1234));
    }

    #[test]
    fn instr_brk_writes_program_counter_and_status_with_break_flag_set_to_stack_pointer() {
        let mut cpu = run_instr(mem!(0x1234 => { BRK }), |cpu| {
            cpu.program_counter = Address::new(0x1234);
            cpu.status = Status::from_bits_truncate(0b1001_1000);
            cpu.stack_pointer.0 = 6;
        });

        assert_eq!(cpu.read(stack::BASE + 6), 0x12);
        assert_eq!(cpu.read(stack::BASE + 5), 0x34);
        assert_eq!(cpu.read(stack::BASE + 4), 0b1011_1000);
    }

    #[test]
    fn instr_brk_decrements_stack_pointer_by_three_bytes() {
        let cpu = run_instr(mem!(BRK), |cpu| {
            cpu.stack_pointer.0 = 6;
        });

        assert_eq!(cpu.stack_pointer.0, 3);
    }

    #[test]
    fn instr_brk_sets_break_flag_on_stack() {
        let mut cpu = run_instr(mem!(BRK), |cpu| {
            cpu.status.remove(Status::BREAK);
            cpu.stack_pointer = StackPointer(6);
        });

        let status = Status::from_bits_truncate(cpu.read(stack::BASE + 4));
        assert!(status.contains(Status::BREAK));
    }

    #[test]
    fn instr_rti_reads_status_and_program_counter_from_stack() {
        let cpu = run_instr(
            mem!(
                0 => { RTI }
                stack::BASE + 101 => { 0x56, 0x34, 0x12 }
            ),
            |cpu| {
                cpu.stack_pointer.0 = 100;
            },
        );

        assert_eq!(cpu.program_counter, Address::new(0x1234));
        assert_eq!(cpu.status.bits(), 0x56);
    }

    #[test]
    fn instr_rti_increments_stack_pointer_by_three_bytes() {
        let cpu = run_instr(mem!(RTI), |cpu| {
            cpu.stack_pointer.0 = 6;
        });

        assert_eq!(cpu.stack_pointer.0, 9);
    }

    #[test]
    fn addition_behaves_appropriately_across_many_values() {
        let carry_values = [true, false];
        let values = [0, 1, 2, 3, 126, 127, 128, 129, 252, 253, 254, 255];

        for x in values.iter() {
            for y in values.iter() {
                for carry_in in carry_values.iter() {
                    let cpu = run_instr(mem!(ADC_IMMEDIATE, *y), |cpu| {
                        cpu.status.set(Status::CARRY, *carry_in);
                        cpu.accumulator = *x;
                    });

                    let carry_bit = *carry_in as u16;
                    let expected = u16::from(*x) + u16::from(*y) + carry_bit;

                    let carry_out = {
                        let this = &cpu;
                        this.status
                    }
                    .contains(Status::CARRY) as u8;
                    let actual = u16::from_be_bytes([carry_out, cpu.accumulator]);

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
                        cpu.status.set(Status::CARRY, *carry_in);
                        cpu.accumulator = *x;
                    });

                    let carry_bit = *carry_in as u16;
                    let expected = (u16::from(*x))
                        .wrapping_sub(u16::from(*y))
                        .wrapping_sub(1 - carry_bit);
                    let expected = expected & 0b1_1111_1111;

                    let carry_out = {
                        let this = &cpu;
                        this.status
                    }
                    .contains(Status::CARRY) as u8;
                    let accumulator = cpu.accumulator;
                    let actual = u16::from_be_bytes([1 - carry_out, accumulator]);

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
            cpu.accumulator = 42;
        });

        assert_eq!(cpu.accumulator, 43);
        assert!(!cpu.status.contains(Status::ZERO));
    }

    #[test]
    fn zero_flag_is_set_when_accumulator_is_zero() {
        let cpu = run_instr(mem!(ADC_IMMEDIATE, 214u8), |cpu| {
            cpu.accumulator = 42;
        });

        assert_eq!(cpu.accumulator, 0);
        assert!(cpu.status.contains(Status::ZERO));
    }

    #[test]
    fn negative_flag_is_not_set_when_accumulator_is_positive() {
        let cpu = run_instr(mem!(ADC_IMMEDIATE, 1u8), |cpu| {
            cpu.accumulator = 42;
        });

        assert_eq!(cpu.accumulator, 43);
        assert!(!cpu.status.contains(Status::NEGATIVE));
    }

    #[test]
    fn negative_flag_is_set_when_accumulator_is_negative() {
        let cpu = run_instr(mem!(ADC_IMMEDIATE, -1i8 as u8), |cpu| {
            cpu.accumulator = 0;
        });

        assert_eq!(cpu.accumulator as i8, -1i8);
        assert!(cpu.status.contains(Status::NEGATIVE));
    }

    #[test]
    fn program_counter_is_incremented_by_1_when_executing_1_byte_instr() {
        let cpu = run_instr(mem!(100 => ASL_ACCUMULATOR), |cpu| {
            cpu.program_counter = Address::new(100)
        });

        assert_eq!(cpu.program_counter, Address::new(101));
    }

    #[test]
    fn program_counter_is_incremented_by_2_when_executing_2_byte_instr() {
        let cpu = run_instr(mem!(100 => { ADC_IMMEDIATE, 0u8 }), |cpu| {
            cpu.program_counter = Address::new(100)
        });

        assert_eq!(cpu.program_counter, Address::new(102));
    }

    #[test]
    fn program_counter_is_incremented_by_3_when_executing_3_byte_instr() {
        let cpu = run_instr(mem!(100 => { ASL_ABSOLUTE, 0, 0 }), |cpu| {
            cpu.program_counter = Address::new(100)
        });

        assert_eq!(cpu.program_counter, Address::new(103));
    }

    #[test]
    fn stack_pointer_wraps_on_overflow() {
        let cpu = run_instr(mem!(PLA), |cpu| {
            cpu.stack_pointer.0 = 255;
        });

        assert_eq!(cpu.stack_pointer.0, 0);

        let cpu = run_instr(mem!(PHA), |cpu| {
            cpu.stack_pointer.0 = 0;
        });

        assert_eq!(cpu.stack_pointer.0, 255);
    }

    #[test]
    fn stack_operations_wrap_value_on_overflow() {
        let mut cpu = run_instr(mem!(0x1234 => { JSR, 100, 0 }), |cpu| {
            cpu.stack_pointer.0 = 0;
            cpu.program_counter = Address::new(0x1234);
        });

        assert_eq!(cpu.read(stack::BASE), 0x12);
        assert_eq!(cpu.read(stack::BASE + 0xff), 0x36);

        let cpu = run_instr(
            mem!(
                40 => { RTS }
                stack::BASE => { 0x12u8 }
                stack::BASE + 0xff => { 0x36u8 }
            ),
            |cpu| {
                cpu.stack_pointer.0 = 0xfe;
                cpu.program_counter = Address::new(40);
            },
        );

        assert_eq!(cpu.program_counter, Address::new(0x1237));
    }

    #[test]
    fn program_counter_wraps_on_overflow() {
        let cpu = run_instr(mem!(0xffff => NOP), |cpu| {
            cpu.program_counter = Address::new(0xffff);
        });

        assert_eq!(cpu.program_counter, Address::new(0));
    }

    #[test]
    fn instructions_can_wrap_on_program_counter_overflow() {
        let cpu = run_instr(mem!(0xfffe => { JMP_ABSOLUTE, 0x34, 0x12 }), |cpu| {
            cpu.program_counter = Address::new(0xfffe);
        });

        assert_eq!(cpu.program_counter, Address::new(0x1234));
    }

    #[test]
    fn on_non_maskable_interrupt_reset_interrupt_flag() {
        let cpu = run_instr(mem!(), |cpu| {
            cpu.non_maskable_interrupt = true;
        });

        assert!(!cpu.non_maskable_interrupt);
    }

    #[test]
    fn on_non_maskable_interrupt_push_program_counter_and_status_with_clear_break_flag_to_stack() {
        let mut cpu = run_instr(mem!(0x1234 => { INX }), |cpu| {
            cpu.program_counter = Address::new(0x1234);
            cpu.status = Status::from_bits_truncate(0b1001_1000);
            cpu.stack_pointer.0 = 6;
            cpu.non_maskable_interrupt = true;
        });

        assert_eq!(cpu.read(stack::BASE + 6), 0x12);
        assert_eq!(cpu.read(stack::BASE + 5), 0x34);
        assert_eq!(cpu.read(stack::BASE + 4), 0b1010_1000);
        assert_eq!(cpu.stack_pointer.0, 3);
    }

    #[test]
    fn on_non_maskable_interrupt_jumps_to_address_at_nmi_vector() {
        let cpu = run_instr(
            mem!(
                0x1234 => { INX }
                0xfffa => { 0x78, 0x56 }
            ),
            |cpu| {
                cpu.program_counter = Address::new(0x1234);
                cpu.non_maskable_interrupt = true;
            },
        );

        assert_eq!(cpu.program_counter, Address::new(0x5678));
    }

    #[test]
    fn calling_non_maskable_interrupt_sets_interrupt_flag() {
        let mut cpu = CPU::from_memory(mem!());
        cpu.non_maskable_interrupt = false;

        cpu.non_maskable_interrupt();

        assert!(cpu.non_maskable_interrupt);
    }

    enum ParameterizedScenario {
        Normal,
        PageCross,
    }
    use ParameterizedScenario::*;

    #[parameterized(
        lda_imm = { LDA(FlexibleAddressingMode::Immediate), 2, Normal },
        lda_zpa = { LDA(FlexibleAddressingMode::ZeroPage), 3, Normal },
        lda_zpx = { LDA(FlexibleAddressingMode::ZeroPageX), 4, Normal },
        lda_abs = { LDA(FlexibleAddressingMode::Absolute), 4, Normal },
        lda_abx = { LDA(FlexibleAddressingMode::AbsoluteX), 4, Normal },
        lda_abx_cross = { LDA(FlexibleAddressingMode::AbsoluteX), 5, PageCross },
        lda_aby = { LDA(FlexibleAddressingMode::AbsoluteY), 4, Normal },
        lda_aby_cross = { LDA(FlexibleAddressingMode::AbsoluteY), 5, PageCross },
        lda_idx = { LDA(FlexibleAddressingMode::IndexedIndirect), 6, Normal },
        lda_idy = { LDA(FlexibleAddressingMode::IndirectIndexed), 5, Normal },
        lda_idy_cross = { LDA(FlexibleAddressingMode::IndirectIndexed), 6, PageCross },

        ldx_imm = { LDX(LDXAddressingMode::Immediate), 2, Normal },
        ldx_zpa = { LDX(LDXAddressingMode::ZeroPage), 3, Normal },
        ldx_zpy = { LDX(LDXAddressingMode::ZeroPageY), 4, Normal },
        ldx_abs = { LDX(LDXAddressingMode::Absolute), 4, Normal },
        ldx_aby = { LDX(LDXAddressingMode::AbsoluteY), 4, Normal },
        ldx_aby_cross = { LDX(LDXAddressingMode::AbsoluteY), 5, PageCross },

        ldy_imm = { LDY(LDYAddressingMode::Immediate), 2, Normal },
        ldy_zpa = { LDY(LDYAddressingMode::ZeroPage), 3, Normal },
        ldy_zpx = { LDY(LDYAddressingMode::ZeroPageX), 4, Normal },
        ldy_abs = { LDY(LDYAddressingMode::Absolute), 4, Normal },
        ldy_abx = { LDY(LDYAddressingMode::AbsoluteX), 4, Normal },
        ldy_abx_cross = { LDY(LDYAddressingMode::AbsoluteX), 5, PageCross },

        sta_zpa = { STA(StoreAddressingMode::ZeroPage), 3, Normal },
        sta_zpx = { STA(StoreAddressingMode::ZeroPageX), 4, Normal },
        sta_abs = { STA(StoreAddressingMode::Absolute), 4, Normal },
        sta_abx = { STA(StoreAddressingMode::AbsoluteX), 5, Normal },
        sta_abx_cross = { STA(StoreAddressingMode::AbsoluteX), 5, PageCross },
        sta_aby = { STA(StoreAddressingMode::AbsoluteY), 5, Normal },
        sta_aby_cross = { STA(StoreAddressingMode::AbsoluteY), 5, PageCross },
        sta_idx = { STA(StoreAddressingMode::IndexedIndirect), 6, Normal },
        sta_idy = { STA(StoreAddressingMode::IndirectIndexed), 6, Normal },
        sta_idy_cross = { STA(StoreAddressingMode::IndirectIndexed), 6, PageCross },

        stx_zpa = { STX(STXAddressingMode::ZeroPage), 3, Normal },
        stx_zpy = { STX(STXAddressingMode::ZeroPageY), 4, Normal },
        stx_abs = { STX(STXAddressingMode::Absolute), 4, Normal },

        sty_zpa = { STY(STYAddressingMode::ZeroPage), 3, Normal },
        sty_zpx = { STY(STYAddressingMode::ZeroPageX), 4, Normal },
        sty_abs = { STY(STYAddressingMode::Absolute), 4, Normal },

        tax = { TAX, 2, Normal },

        tay = { TAY, 2, Normal },

        txa = { TXA, 2, Normal },

        tya = { TYA, 2, Normal },

        tsx = { TSX, 2, Normal },

        txs = { TXS, 2, Normal },

        pha = { PHA, 3, Normal },

        php = { PHP, 3, Normal },

        pla = { PLA, 4, Normal },

        plp = { PLP, 4, Normal },

        and_imm = { AND(FlexibleAddressingMode::Immediate), 2, Normal },
        and_zpa = { AND(FlexibleAddressingMode::ZeroPage), 3, Normal },
        and_zpx = { AND(FlexibleAddressingMode::ZeroPageX), 4, Normal },
        and_abs = { AND(FlexibleAddressingMode::Absolute), 4, Normal },
        and_abx = { AND(FlexibleAddressingMode::AbsoluteX), 4, Normal },
        and_abx_cross = { AND(FlexibleAddressingMode::AbsoluteX), 5, PageCross },
        and_aby = { AND(FlexibleAddressingMode::AbsoluteY), 4, Normal },
        and_aby_cross = { AND(FlexibleAddressingMode::AbsoluteY), 5, PageCross },
        and_idx = { AND(FlexibleAddressingMode::IndexedIndirect), 6, Normal },
        and_idy = { AND(FlexibleAddressingMode::IndirectIndexed), 5, Normal },
        and_idy_cross = { AND(FlexibleAddressingMode::IndirectIndexed), 6, PageCross },

        eor_imm = { EOR(FlexibleAddressingMode::Immediate), 2, Normal },
        eor_zpa = { EOR(FlexibleAddressingMode::ZeroPage), 3, Normal },
        eor_zpx = { EOR(FlexibleAddressingMode::ZeroPageX), 4, Normal },
        eor_abs = { EOR(FlexibleAddressingMode::Absolute), 4, Normal },
        eor_abx = { EOR(FlexibleAddressingMode::AbsoluteX), 4, Normal },
        eor_abx_cross = { EOR(FlexibleAddressingMode::AbsoluteX), 5, PageCross },
        eor_aby = { EOR(FlexibleAddressingMode::AbsoluteY), 4, Normal },
        eor_aby_cross = { EOR(FlexibleAddressingMode::AbsoluteY), 5, PageCross },
        eor_idx = { EOR(FlexibleAddressingMode::IndexedIndirect), 6, Normal },
        eor_idy = { EOR(FlexibleAddressingMode::IndirectIndexed), 5, Normal },
        eor_idy_cross = { EOR(FlexibleAddressingMode::IndirectIndexed), 6, PageCross },

        ora_imm = { ORA(FlexibleAddressingMode::Immediate), 2, Normal },
        ora_zpa = { ORA(FlexibleAddressingMode::ZeroPage), 3, Normal },
        ora_zpx = { ORA(FlexibleAddressingMode::ZeroPageX), 4, Normal },
        ora_abs = { ORA(FlexibleAddressingMode::Absolute), 4, Normal },
        ora_abx = { ORA(FlexibleAddressingMode::AbsoluteX), 4, Normal },
        ora_abx_cross = { ORA(FlexibleAddressingMode::AbsoluteX), 5, PageCross },
        ora_aby = { ORA(FlexibleAddressingMode::AbsoluteY), 4, Normal },
        ora_aby_cross = { ORA(FlexibleAddressingMode::AbsoluteY), 5, PageCross },
        ora_idx = { ORA(FlexibleAddressingMode::IndexedIndirect), 6, Normal },
        ora_idy = { ORA(FlexibleAddressingMode::IndirectIndexed), 5, Normal },
        ora_idy_cross = { ORA(FlexibleAddressingMode::IndirectIndexed), 6, PageCross },

        bit_zpa = { BIT(BITAddressingMode::ZeroPage), 3, Normal },
        bit_abs = { BIT(BITAddressingMode::Absolute), 4, Normal },

        adc_imm = { ADC(FlexibleAddressingMode::Immediate), 2, Normal },
        adc_zpa = { ADC(FlexibleAddressingMode::ZeroPage), 3, Normal },
        adc_zpx = { ADC(FlexibleAddressingMode::ZeroPageX), 4, Normal },
        adc_abs = { ADC(FlexibleAddressingMode::Absolute), 4, Normal },
        adc_abx = { ADC(FlexibleAddressingMode::AbsoluteX), 4, Normal },
        adc_abx_cross = { ADC(FlexibleAddressingMode::AbsoluteX), 5, PageCross },
        adc_aby = { ADC(FlexibleAddressingMode::AbsoluteY), 4, Normal },
        adc_aby_cross = { ADC(FlexibleAddressingMode::AbsoluteY), 5, PageCross },
        adc_idx = { ADC(FlexibleAddressingMode::IndexedIndirect), 6, Normal },
        adc_idy = { ADC(FlexibleAddressingMode::IndirectIndexed), 5, Normal },
        adc_idy_cross = { ADC(FlexibleAddressingMode::IndirectIndexed), 6, PageCross },

        sbc_imm = { SBC(FlexibleAddressingMode::Immediate), 2, Normal },
        sbc_zpa = { SBC(FlexibleAddressingMode::ZeroPage), 3, Normal },
        sbc_zpx = { SBC(FlexibleAddressingMode::ZeroPageX), 4, Normal },
        sbc_abs = { SBC(FlexibleAddressingMode::Absolute), 4, Normal },
        sbc_abx = { SBC(FlexibleAddressingMode::AbsoluteX), 4, Normal },
        sbc_abx_cross = { SBC(FlexibleAddressingMode::AbsoluteX), 5, PageCross },
        sbc_aby = { SBC(FlexibleAddressingMode::AbsoluteY), 4, Normal },
        sbc_aby_cross = { SBC(FlexibleAddressingMode::AbsoluteY), 5, PageCross },
        sbc_idx = { SBC(FlexibleAddressingMode::IndexedIndirect), 6, Normal },
        sbc_idy = { SBC(FlexibleAddressingMode::IndirectIndexed), 5, Normal },
        sbc_idy_cross = { SBC(FlexibleAddressingMode::IndirectIndexed), 6, PageCross },

        cmp_imm = { CMP(FlexibleAddressingMode::Immediate), 2, Normal },
        cmp_zpa = { CMP(FlexibleAddressingMode::ZeroPage), 3, Normal },
        cmp_zpx = { CMP(FlexibleAddressingMode::ZeroPageX), 4, Normal },
        cmp_abs = { CMP(FlexibleAddressingMode::Absolute), 4, Normal },
        cmp_abx = { CMP(FlexibleAddressingMode::AbsoluteX), 4, Normal },
        cmp_abx_cross = { CMP(FlexibleAddressingMode::AbsoluteX), 5, PageCross },
        cmp_aby = { CMP(FlexibleAddressingMode::AbsoluteY), 4, Normal },
        cmp_aby_cross = { CMP(FlexibleAddressingMode::AbsoluteY), 5, PageCross },
        cmp_idx = { CMP(FlexibleAddressingMode::IndexedIndirect), 6, Normal },
        cmp_idy = { CMP(FlexibleAddressingMode::IndirectIndexed), 5, Normal },
        cmp_idy_cross = { CMP(FlexibleAddressingMode::IndirectIndexed), 6, PageCross },

        cpx_imm = { CPX(CompareAddressingMode::Immediate), 2, Normal },
        cpx_zpa = { CPX(CompareAddressingMode::ZeroPage), 3, Normal },
        cpx_abs = { CPX(CompareAddressingMode::Absolute), 4, Normal },

        cpy_imm = { CPY(CompareAddressingMode::Immediate), 2, Normal },
        cpy_zpa = { CPY(CompareAddressingMode::ZeroPage), 3, Normal },
        cpy_abs = { CPY(CompareAddressingMode::Absolute), 4, Normal },

        inc_zpa = { INC(IncDecAddressingMode::ZeroPage), 5, Normal },
        inc_zpx = { INC(IncDecAddressingMode::ZeroPageX), 6, Normal },
        inc_abs = { INC(IncDecAddressingMode::Absolute), 6, Normal },
        inc_abx = { INC(IncDecAddressingMode::AbsoluteX), 7, Normal },
        inc_abx_cross = { INC(IncDecAddressingMode::AbsoluteX), 7, PageCross },

        inx = { INX, 2, Normal },

        iny = { INY, 2, Normal },

        dec_zpa = { DEC(IncDecAddressingMode::ZeroPage), 5, Normal },
        dec_zpx = { DEC(IncDecAddressingMode::ZeroPageX), 6, Normal },
        dec_abs = { DEC(IncDecAddressingMode::Absolute), 6, Normal },
        dec_abx = { DEC(IncDecAddressingMode::AbsoluteX), 7, Normal },
        dec_abx_cross = { DEC(IncDecAddressingMode::AbsoluteX), 7, PageCross },

        dex = { DEX, 2, Normal },

        dey = { DEY, 2, Normal },

        asl_acc = { ASL(ShiftAddressingMode::Accumulator), 2, Normal },
        asl_zpa = { ASL(ShiftAddressingMode::ZeroPage), 5, Normal },
        asl_zpx = { ASL(ShiftAddressingMode::ZeroPageX), 6, Normal },
        asl_abs = { ASL(ShiftAddressingMode::Absolute), 6, Normal },
        asl_abx = { ASL(ShiftAddressingMode::AbsoluteX), 7, Normal },
        asl_abx_cross = { ASL(ShiftAddressingMode::AbsoluteX), 7, PageCross },

        lsr_acc = { LSR(ShiftAddressingMode::Accumulator), 2, Normal },
        lsr_zpa = { LSR(ShiftAddressingMode::ZeroPage), 5, Normal },
        lsr_zpx = { LSR(ShiftAddressingMode::ZeroPageX), 6, Normal },
        lsr_abs = { LSR(ShiftAddressingMode::Absolute), 6, Normal },
        lsr_abx = { LSR(ShiftAddressingMode::AbsoluteX), 7, Normal },
        lsr_abx_cross = { LSR(ShiftAddressingMode::AbsoluteX), 7, PageCross },

        rol_acc = { ROL(ShiftAddressingMode::Accumulator), 2, Normal },
        rol_zpa = { ROL(ShiftAddressingMode::ZeroPage), 5, Normal },
        rol_zpx = { ROL(ShiftAddressingMode::ZeroPageX), 6, Normal },
        rol_abs = { ROL(ShiftAddressingMode::Absolute), 6, Normal },
        rol_abx = { ROL(ShiftAddressingMode::AbsoluteX), 7, Normal },
        rol_abx_cross = { ROL(ShiftAddressingMode::AbsoluteX), 7, PageCross },

        ror_acc = { ROR(ShiftAddressingMode::Accumulator), 2, Normal },
        ror_zpa = { ROR(ShiftAddressingMode::ZeroPage), 5, Normal },
        ror_zpx = { ROR(ShiftAddressingMode::ZeroPageX), 6, Normal },
        ror_abs = { ROR(ShiftAddressingMode::Absolute), 6, Normal },
        ror_abx = { ROR(ShiftAddressingMode::AbsoluteX), 7, Normal },
        ror_abx_cross = { ROR(ShiftAddressingMode::AbsoluteX), 7, PageCross },

        jmp_abs = { JMP(JumpAddressingMode::Absolute), 3, Normal },
        jmp_ind = { JMP(JumpAddressingMode::Indirect), 5, Normal },

        jsr = { JSR, 6, Normal },

        rts = { RTS, 6, Normal },

        // TODO: branch failure/success/page cases
        bcc = { BCC, 3, Normal },
        bcc_cross = { BCC, 4, PageCross },

        bcs = { BCS, 2, Normal },
        bcs_cross = { BCS, 2, PageCross },

        beq = { BEQ, 2, Normal },
        beq_cross = { BEQ, 2, PageCross },

        bmi = { BMI, 2, Normal },
        bmi_cross = { BMI, 2, PageCross },

        bne = { BNE, 3, Normal },
        bne_cross = { BNE, 4, PageCross },

        bpl = { BPL, 3, Normal },
        bpl_cross = { BPL, 4, PageCross },

        bvc = { BVC, 3, Normal },
        bvc_cross = { BVC, 4, PageCross },

        bvs = { BVS, 2, Normal },
        bvs_cross = { BVS, 2, PageCross },

        clc = { CLC, 2, Normal },

        cld = { CLD, 2, Normal },

        cli = { CLI, 2, Normal },

        clv = { CLV, 2, Normal },

        sec = { SEC, 2, Normal },

        sed = { SED, 2, Normal },

        sei = { SEI, 2, Normal },

        brk = { BRK, 7, Normal },

        nop = { NOP, 2, Normal },

        rti = { RTI, 6, Normal },
    )]
    fn basic_instructions_return_correct_number_of_cycles(
        instruction: Instruction,
        expected_cycles: u8,
        scenario: ParameterizedScenario,
    ) {
        let mut cpu = CPU::from_memory(mem!(instruction));

        match scenario {
            Normal => {}
            PageCross => {
                // Make sure a page cross happens with any addressing mode
                cpu.write(Address::new(0x01), 0x80);
                cpu.write(Address::new(0x80), 0xFF);
                cpu.x = 0xFF;
                cpu.y = 0xFF;
            }
        };

        let actual_cycles = cpu.run_instruction();
        assert_eq!(actual_cycles, expected_cycles, "{:?}", instruction);
    }

    #[test]
    fn instruction_sequence_return_correct_number_of_cycles() {
        let start = Address::new(0xE084);
        let foo_zero_addr = 0x10;
        let foo_addr = Address::from_bytes(0, foo_zero_addr);
        let foo_init = 0xFE; // Nearly overflowing

        // Instructions from blargg_cpu_tests, not meaningful
        let mut cpu = CPU::from_memory(mem!(
            RESET_VECTOR => { start.lower(), start.higher() }
            foo_addr => { foo_init }
            start.bytes() => {
                CPX_ZERO_PAGE, 0x12,
                BNE, 9,
                INC_ZERO_PAGE, foo_zero_addr,
                BNE, (-8i8) as u8,
                INC_ZERO_PAGE, 0x11,
                JMP_ABSOLUTE, start.lower(), start.higher()
            }
        ));

        assert_eq!(cpu.program_counter, start);

        // CPX
        let cycles = cpu.run_instruction();
        assert_eq!(cycles, 3);
        assert_eq!(cpu.program_counter, start + 2);

        // BNE
        let cycles = cpu.run_instruction();
        assert_eq!(cycles, 2, "BNE should be 2 cycles if it doesn't branch");
        assert_eq!(cpu.program_counter, start + 4);

        // INC
        let cycles = cpu.run_instruction();
        assert_eq!(cycles, 5);
        assert_eq!(cpu.program_counter, start + 6);
        assert_eq!(cpu.memory.read(foo_addr), foo_init + 1);

        // BNE (jump to start because no overflow)
        let cycles = cpu.run_instruction();
        assert_eq!(cycles, 3, "BNE should be 3 cycles if it branches");
        assert_eq!(cpu.program_counter, start);

        // Run same instructions again
        cpu.run_instruction(); // CPX
        cpu.run_instruction(); // BNE

        // INC (overflows)
        let cycles = cpu.run_instruction();
        assert_eq!(cycles, 5);
        assert_eq!(cpu.program_counter, start + 6);
        assert_eq!(cpu.memory.read(foo_addr), 0, "INC should overflow");

        // BNE (don't jump because overflow)
        let cycles = cpu.run_instruction();
        assert_eq!(cycles, 2, "BNE should be 2 cycles if it doesn't branch");
        assert_eq!(cpu.program_counter, start + 8);

        // INC
        let cycles = cpu.run_instruction();
        assert_eq!(cycles, 5);
        assert_eq!(cpu.program_counter, start + 10);

        // JMP
        let cycles = cpu.run_instruction();
        assert_eq!(cycles, 3);
        assert_eq!(cpu.program_counter, start);
    }

    pub fn run_instr<F: FnOnce(&mut CPU<ArrayMemory>)>(
        memory: ArrayMemory,
        cpu_setup: F,
    ) -> CPU<ArrayMemory> {
        let mut cpu = CPU::from_memory(memory);

        cpu_setup(&mut cpu);

        cpu.run_instruction();

        hexdump::hexdump(&cpu.memory.slice()[..0x200]);

        cpu
    }
}
