//! Jumps & Calls

use crate::cpu::Tickable;
use crate::{cpu::addressing_modes::JumpAddressingMode, Address, Memory, CPU};

impl<M: Memory + Tickable> CPU<'_, M> {
    pub(in crate::cpu) fn jmp(&mut self, addressing_mode: JumpAddressingMode) {
        self.state.program_counter = addressing_mode.fetch_address(self);
    }

    pub(in crate::cpu) fn jsr(&mut self) {
        let addr = self.fetch_address_at_program_counter();

        self.increment_cycle_count(); // Mysterious internal operation happens here

        // For some reason the spec says the pointer must be to the last byte of the JSR
        // instruction...
        let data = self.state.program_counter - 1;

        self.push_stack(data.higher());
        self.push_stack(data.lower());

        self.state.program_counter = addr;
    }

    pub(in crate::cpu) fn rts(&mut self) {
        self.ignore_argument();
        self.increment_stack();
        let lower = self.pull_and_increment_stack();
        let higher = self.pull_stack();
        self.state.program_counter = Address::from_bytes(higher, lower);
        self.incr_program_counter();
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::{
            stack::{self},
            tests::run_instr,
        }, instructions::{JMP_ABS, JMP_IND, JSR, RTS},
        mem,
        Address,
        Memory,
    };

    #[test]
    fn instr_jmp_jumps_to_immediate_operand() {
        let cpu = run_instr(mem!(200 => { JMP_ABS, 100, 0 }), |cpu| {
            cpu.program_counter = Address::new(200);
        });

        assert_eq!(cpu.state.program_counter, Address::new(100));
    }

    #[test]
    fn instr_jmp_jumps_to_indirect_operand() {
        let cpu = run_instr(
            mem!(
                20 => { JMP_IND, 30, 0 }
                30 => { 10, 0 }
            ),
            |cpu| {
                cpu.program_counter = Address::new(20);
            },
        );

        assert_eq!(cpu.state.program_counter, Address::new(10));
    }

    #[test]
    fn instr_jsr_jumps_to_operand() {
        let cpu = run_instr(mem!(200 => { JSR, 100, 0 }), |cpu| {
            cpu.program_counter = Address::new(200);
        });

        assert_eq!(cpu.state.program_counter, Address::new(100));
    }

    #[test]
    fn instr_jsr_writes_program_counter_to_stack_pointer() {
        let mut cpu = run_instr(mem!(0x1234 => { JSR, 100, 0 }), |cpu| {
            cpu.program_counter = Address::new(0x1234);
            cpu.stack_pointer.0 = 6;
        });

        // Program counter points to last byte of JSR instruction
        assert_eq!(cpu.memory.read(stack::BASE + 6), 0x12);
        assert_eq!(cpu.memory.read(stack::BASE + 5), 0x36);
    }

    #[test]
    fn instr_jsr_decrements_stack_pointer_by_two_bytes() {
        let cpu = run_instr(mem!(JSR, 0x23, 0x01), |cpu| {
            cpu.stack_pointer.0 = 6;
        });

        assert_eq!(cpu.state.stack_pointer.0, 4);
    }

    #[test]
    fn instr_rts_reads_program_counter_plus_one_from_stack() {
        let cpu = run_instr(
            mem!(
                0 => { RTS }
                stack::BASE + 101 => { 0x34, 0x12 }
            ),
            |cpu| {
                cpu.stack_pointer.0 = 100;
            },
        );

        assert_eq!(cpu.state.program_counter, Address::new(0x1235));
    }

    #[test]
    fn instr_rts_increments_stack_pointer_by_two_bytes() {
        let cpu = run_instr(mem!(RTS), |cpu| {
            cpu.stack_pointer.0 = 6;
        });

        assert_eq!(cpu.state.stack_pointer.0, 8);
    }
}
