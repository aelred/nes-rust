//! Logical operations

use crate::{
    cpu::{
        addressing_modes::{BITAddressingMode, FlexibleAddressingMode},
        Status,
    },
    Memory, CPU,
};

impl<M: Memory> CPU<M> {
    pub fn and(&mut self, addressing_mode: FlexibleAddressingMode) {
        let value = self.fetch(addressing_mode);
        self.set_accumulator(self.accumulator & value);
    }

    pub fn eor(&mut self, addressing_mode: FlexibleAddressingMode) {
        let value = self.fetch(addressing_mode);
        self.set_accumulator(self.accumulator ^ value);
    }

    pub fn ora(&mut self, addressing_mode: FlexibleAddressingMode) {
        let value = self.fetch(addressing_mode);
        self.set_accumulator(self.accumulator | value);
    }

    pub fn bit(&mut self, addressing_mode: BITAddressingMode) {
        let value = self.fetch(addressing_mode);
        let result = self.accumulator & value;
        self.status.set(Status::ZERO, result == 0);
        self.status.set(Status::OVERFLOW, value & (1 << 6) != 0);
        self.status
            .set(Status::NEGATIVE, (value as i8).is_negative());
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::{tests::run_instr, Status},
        instructions::{AND_IMMEDIATE, BIT_ABSOLUTE, EOR_IMMEDIATE, ORA_IMMEDIATE},
        mem,
    };

    #[test]
    fn instr_and_performs_bitwise_and() {
        let cpu = run_instr(mem!(AND_IMMEDIATE, 0b1100_u8), |cpu| {
            cpu.accumulator = 0b1010;
        });

        assert_eq!(cpu.accumulator, 0b1000);
    }

    #[test]
    fn instr_eor_performs_bitwise_xor() {
        let cpu = run_instr(mem!(EOR_IMMEDIATE, 0b1100_u8), |cpu| {
            cpu.accumulator = 0b1010;
        });

        assert_eq!(cpu.accumulator, 0b0110);
    }

    #[test]
    fn instr_ora_performs_bitwise_or() {
        let cpu = run_instr(mem!(ORA_IMMEDIATE, 0b1100_u8), |cpu| {
            cpu.accumulator = 0b1010;
        });

        assert_eq!(cpu.accumulator, 0b1110);
    }

    #[test]
    fn instr_bit_sets_zero_flag_when_bitwise_and_is_zero() {
        let cpu = run_instr(
            mem!(
                0 => { BIT_ABSOLUTE, 54, 0 }
                54 => { 0b0000_1111 }
            ),
            |cpu| {
                cpu.accumulator = 0b1111_0000u8;
            },
        );

        assert!(cpu.status.contains(Status::ZERO));
    }

    #[test]
    fn instr_bit_clears_zero_flag_when_bitwise_and_is_not_zero() {
        let cpu = run_instr(
            mem!(
                0 => { BIT_ABSOLUTE, 54, 0 }
                54 => { 0b0011_1111 }
            ),
            |cpu| {
                cpu.accumulator = 0b1111_1100u8;
            },
        );

        assert!(!cpu.status.contains(Status::ZERO));
    }

    #[test]
    fn instr_bit_sets_overflow_bit_based_on_bit_6_of_operand() {
        let cpu = run_instr(
            mem!(
                0 => { BIT_ABSOLUTE, 54, 0 }
                54 => { 0 }
            ),
            |_| {},
        );

        assert!(!cpu.status.contains(Status::OVERFLOW));

        let cpu = run_instr(
            mem!(
                0 => { BIT_ABSOLUTE, 54, 0 }
                54 => { 0b0100_0000 }
            ),
            |_| {},
        );

        assert!(cpu.status.contains(Status::OVERFLOW));
    }

    #[test]
    fn instr_bit_sets_negative_bit_based_on_bit_7_of_operand() {
        let cpu = run_instr(
            mem!(
                0 => { BIT_ABSOLUTE, 54, 0 }
                54 => { 0 }
            ),
            |_| {},
        );

        assert!(!cpu.status.contains(Status::NEGATIVE));

        let cpu = run_instr(
            mem!(
                0 => { BIT_ABSOLUTE, 54, 0 }
                54 => { 0b1000_0000 }
            ),
            |_| {},
        );

        assert!(cpu.status.contains(Status::NEGATIVE));
    }
}
