//! System Functions

use crate::{
    cpu::{addressing_modes::IncDecAddressingMode, Status},
    Address, Memory, CPU,
};

const INTERRUPT_VECTOR: Address = Address::new(0xFFFE);

impl<M: Memory> CPU<M> {
    pub(in crate::cpu) fn brk(&mut self) {
        self.ignore_argument();
        self.interrupt(INTERRUPT_VECTOR, true)
    }

    pub(in crate::cpu) fn nop(&mut self) {
        self.ignore_argument();
    }

    pub(in crate::cpu) fn rti(&mut self) {
        self.ignore_argument();
        self.increment_stack();
        self.status = Status::from_bits_truncate(self.pull_and_increment_stack());
        let lower = self.pull_and_increment_stack();
        let higher = self.pull_stack();
        self.program_counter = Address::from_bytes(higher, lower);
    }

    // Unofficial Opcodes
    pub(in crate::cpu) fn ign(&mut self, addressing_mode: IncDecAddressingMode) {
        self.fetch_ref(addressing_mode);
    }

    pub(in crate::cpu) fn skb(&mut self) {
        self.incr_program_counter();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cpu::{stack, tests::run_instr, Status},
        instructions::{BRK, LSR_ACCUMULATOR, RTI},
        mem, Address,
    };

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
            cpu.stack_pointer.0 = 6;
        });

        let status = Status::from_bits_truncate(cpu.read(stack::BASE + 4));
        assert!(status.contains(Status::BREAK));
    }

    #[test]
    fn instr_nop_increments_program_counter() {
        let cpu = run_instr(mem!(20 => LSR_ACCUMULATOR), |cpu| {
            cpu.program_counter = Address::new(20);
        });

        assert_eq!(cpu.program_counter, Address::new(21));
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
}
