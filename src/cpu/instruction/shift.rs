//! Shifts

use crate::{
    cpu::{addressing_modes::StoreAddressingMode, ReferenceAddressingMode, Status},
    Memory, CPU,
};

impl<M: Memory> CPU<M> {
    pub(in crate::cpu) fn asl(&mut self, addressing_mode: impl ReferenceAddressingMode) -> u8 {
        self.shift(addressing_mode, 7, |val, _| val << 1)
    }

    pub(in crate::cpu) fn lsr(&mut self, addressing_mode: impl ReferenceAddressingMode) -> u8 {
        self.shift(addressing_mode, 0, |val, _| val >> 1)
    }

    pub(in crate::cpu) fn rol(&mut self, addressing_mode: impl ReferenceAddressingMode) -> u8 {
        self.shift(addressing_mode, 7, |val, carry| val << 1 | carry)
    }

    pub(in crate::cpu) fn ror(&mut self, addressing_mode: impl ReferenceAddressingMode) -> u8 {
        self.shift(addressing_mode, 0, |val, carry| val >> 1 | carry << 7)
    }

    // Unofficial Opcodes
    pub(in crate::cpu) fn slo(&mut self, addressing_mode: StoreAddressingMode) {
        let value = self.asl(addressing_mode);
        self.set_accumulator(self.accumulator | value);
    }

    pub(in crate::cpu) fn rla(&mut self, addressing_mode: StoreAddressingMode) {
        let value = self.rol(addressing_mode);
        self.set_accumulator(self.accumulator & value);
    }

    pub(in crate::cpu) fn sre(&mut self, addressing_mode: StoreAddressingMode) {
        let value = self.lsr(addressing_mode);
        self.set_accumulator(self.accumulator ^ value);
    }

    pub(in crate::cpu) fn rra(&mut self, addressing_mode: StoreAddressingMode) {
        let value = self.ror(addressing_mode);
        self.add_to_accumulator(value);
    }

    fn shift(
        &mut self,
        addressing_mode: impl ReferenceAddressingMode,
        carry_bit: u8,
        op: impl FnOnce(u8, u8) -> u8,
    ) -> u8 {
        let reference = self.fetch_ref(addressing_mode);
        let carry = self.status.contains(Status::CARRY);

        let old_value = self.read_reference(reference, false);
        self.set_reference(reference, old_value, false); // Redundant write
        let new_value = op(old_value, carry as u8);
        let carry = old_value & (1 << carry_bit) != 0;

        self.set_reference(reference, new_value, false);
        self.status.set(Status::CARRY, carry);
        new_value
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::{tests::run_instr, Status},
        instructions::{
            ASL_ABSOLUTE, ASL_ACCUMULATOR, LSR_ACCUMULATOR, ROL_ACCUMULATOR, ROR_ACCUMULATOR,
        },
        mem, Address,
    };

    #[test]
    fn instr_asl_shifts_left() {
        let cpu = run_instr(mem!(ASL_ACCUMULATOR), |cpu| {
            cpu.accumulator = 0b100;
        });

        assert_eq!(cpu.accumulator, 0b1000);
        assert!(!cpu.status.contains(Status::CARRY));
    }

    #[test]
    fn instr_asl_sets_carry_flag_on_overflow() {
        let cpu = run_instr(mem!(ASL_ACCUMULATOR), |cpu| {
            cpu.accumulator = 0b1010_1010;
        });

        assert_eq!(cpu.accumulator, 0b0101_0100);
        assert!(cpu.status.contains(Status::CARRY));
    }

    #[test]
    fn instr_asl_can_operate_on_memory() {
        let mut cpu = run_instr(
            mem!(
                0 => { ASL_ABSOLUTE, 100, 0 }
                100 => { 0b100 }
            ),
            |_| {},
        );

        assert_eq!(cpu.read(Address::new(100)), 0b1000);
    }

    #[test]
    fn instr_lsr_shifts_right() {
        let cpu = run_instr(mem!(LSR_ACCUMULATOR), |cpu| {
            cpu.accumulator = 0b100;
        });

        assert_eq!(cpu.accumulator, 0b10);
        assert!(!cpu.status.contains(Status::CARRY));
    }

    #[test]
    fn instr_lsr_sets_carry_flag_on_underflow() {
        let cpu = run_instr(mem!(LSR_ACCUMULATOR), |cpu| {
            cpu.accumulator = 0b101_0101;
        });

        assert_eq!(cpu.accumulator, 0b10_1010);
        assert!(cpu.status.contains(Status::CARRY));
    }

    #[test]
    fn instr_rol_rotates_left_with_carry_flag() {
        let cpu = run_instr(mem!(ROL_ACCUMULATOR), |cpu| {
            cpu.status.remove(Status::CARRY);
            cpu.accumulator = 0b100;
        });

        assert_eq!(cpu.accumulator, 0b1000);
        assert!(!cpu.status.contains(Status::CARRY));

        let cpu = run_instr(mem!(ROL_ACCUMULATOR), |cpu| {
            cpu.status.insert(Status::CARRY);
            cpu.accumulator = 0b100;
        });

        assert_eq!(cpu.accumulator, 0b1001);
        assert!(!cpu.status.contains(Status::CARRY));

        let cpu = run_instr(mem!(ROL_ACCUMULATOR), |cpu| {
            cpu.status.remove(Status::CARRY);
            cpu.accumulator = 0b1000_0000;
        });

        assert_eq!(cpu.accumulator, 0);
        assert!(cpu.status.contains(Status::CARRY));
    }

    #[test]
    fn instr_ror_rotates_left_with_carry_flag() {
        let cpu = run_instr(mem!(ROR_ACCUMULATOR), |cpu| {
            cpu.status.remove(Status::CARRY);
            cpu.accumulator = 0b100;
        });

        assert_eq!(cpu.accumulator, 0b10);
        assert!(!cpu.status.contains(Status::CARRY));

        let cpu = run_instr(mem!(ROR_ACCUMULATOR), |cpu| {
            cpu.status.insert(Status::CARRY);
            cpu.accumulator = 0b100;
        });

        assert_eq!(cpu.accumulator, 0b1000_0010);
        assert!(!cpu.status.contains(Status::CARRY));

        let cpu = run_instr(mem!(ROR_ACCUMULATOR), |cpu| {
            cpu.status.remove(Status::CARRY);
            cpu.accumulator = 0b1;
        });

        assert_eq!(cpu.accumulator, 0);
        assert!(cpu.status.contains(Status::CARRY));
    }
}
