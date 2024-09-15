use crate::Memory;

use super::{Status, CPU};

impl<M: Memory> CPU<M> {
    pub fn pla(&mut self) {
        self.fetch_at_program_counter();
        self.increment_stack();
        let accumulator = self.pull_stack();
        self.set_accumulator(accumulator);
    }

    pub fn plp(&mut self) {
        self.fetch_at_program_counter();
        self.increment_stack();
        self.status = Status::from_bits_truncate(self.pull_stack());
    }

    pub fn pha(&mut self) {
        self.fetch_at_program_counter();
        self.push_stack(self.accumulator)
    }

    pub fn php(&mut self) {
        self.fetch_at_program_counter();
        self.push_status(true)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::{stack, tests::run_instr, Status},
        instructions::{PHA, PHP, PLA, PLP},
        mem,
    };

    #[test]
    fn instr_pla_reads_accumulator_from_stack() {
        let cpu = run_instr(
            mem!(
                0 => { PLA }
                stack::BASE + 7 => { 20 }
            ),
            |cpu| {
                cpu.stack_pointer.0 = 6;
            },
        );

        assert_eq!(cpu.accumulator, 20);
    }

    #[test]
    fn instr_pla_increments_stack_pointer_by_one_byte() {
        let cpu = run_instr(mem!(PLA), |cpu| {
            cpu.stack_pointer.0 = 6;
        });

        assert_eq!(cpu.stack_pointer.0, 7);
    }

    #[test]
    fn instr_plp_reads_status_from_stack() {
        let cpu = run_instr(
            mem!(
                0 => { PLP }
                stack::BASE => { 31 }
            ),
            |_| {},
        );

        assert_eq!(cpu.status.bits(), 31);
    }

    #[test]
    fn instr_plp_increments_stack_pointer_by_one_byte() {
        let cpu = run_instr(mem!(PLP), |cpu| {
            cpu.stack_pointer.0 = 6;
        });

        assert_eq!(cpu.stack_pointer.0, 7);
    }

    #[test]
    fn instr_pha_writes_accumulator_to_stack_pointer() {
        let mut cpu = run_instr(mem!(PHA), |cpu| {
            cpu.accumulator = 20;
            cpu.stack_pointer.0 = 6;
        });

        assert_eq!(cpu.read(stack::BASE + 6), 20);
    }

    #[test]
    fn instr_pha_decrements_stack_pointer_by_one_byte() {
        let cpu = run_instr(mem!(PHA), |cpu| {
            cpu.stack_pointer.0 = 6;
        });

        assert_eq!(cpu.stack_pointer.0, 5);
    }

    #[test]
    fn instr_php_writes_status_to_stack_pointer_with_break_always_set() {
        let mut cpu = run_instr(mem!(PHP), |cpu| {
            cpu.status = Status::from_bits_truncate(0b1100_0101);
            cpu.stack_pointer.0 = 6;
        });

        assert_eq!(cpu.read(stack::BASE + 6), 0b1111_0101);
    }

    #[test]
    fn instr_php_decrements_stack_pointer_by_one_byte() {
        let cpu = run_instr(mem!(PHP), |cpu| {
            cpu.stack_pointer.0 = 6;
        });

        assert_eq!(cpu.stack_pointer.0, 5);
    }
}
