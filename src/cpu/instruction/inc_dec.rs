//! Increments & Decrements

use crate::{
    cpu::{
        addressing_modes::{IncDecAddressingMode, StoreAddressingMode},
        Reference,
    },
    Memory, CPU,
};

impl<M: Memory> CPU<M> {
    pub(in crate::cpu) fn inc(&mut self, addressing_mode: IncDecAddressingMode) {
        let reference = self.fetch_ref(addressing_mode);
        self.increment(reference);
    }

    pub(in crate::cpu) fn inx(&mut self) {
        self.ignore_argument();
        self.increment(Reference::X);
    }

    pub(in crate::cpu) fn iny(&mut self) {
        self.ignore_argument();
        self.increment(Reference::Y);
    }

    pub(in crate::cpu) fn dec(&mut self, addressing_mode: IncDecAddressingMode) {
        let reference = self.fetch_ref(addressing_mode);
        self.decrement(reference);
    }

    pub(in crate::cpu) fn dex(&mut self) {
        self.ignore_argument();
        self.decrement(Reference::X);
    }

    pub(in crate::cpu) fn dey(&mut self) {
        self.ignore_argument();
        self.decrement(Reference::Y);
    }

    // Unofficial Opcodes
    pub(in crate::cpu) fn dcp(&mut self, addressing_mode: StoreAddressingMode) {
        let reference = self.fetch_ref(addressing_mode);
        self.decrement(reference);
        let value = self.read_reference(reference, false);
        self.compare(self.accumulator, value);
    }

    pub(in crate::cpu) fn isc(&mut self, addressing_mode: StoreAddressingMode) {
        let reference = self.fetch_ref(addressing_mode);
        self.increment(reference);
        let value = self.read_reference(reference, false);
        self.sub_from_accumulator(value);
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
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::{tests::run_instr, Status},
        instructions::{DEC_ABS, DEX, DEY, INC_ABS, INX, INY},
        mem, Address,
    };

    #[test]
    fn instr_inc_increments_operand() {
        let mut cpu = run_instr(
            mem!(
                0 => { INC_ABS, 100, 0 }
                100 => { 45 }
            ),
            |_| {},
        );

        assert_eq!(cpu.read(Address::new(100)), 46);
    }

    #[test]
    fn instr_inc_sets_zero_flag_based_on_result() {
        let mut cpu = run_instr(
            mem!(
                0 => { INC_ABS, 100, 0 }
                100 => { 45 }
            ),
            |_| {},
        );

        assert_eq!(cpu.read(Address::new(100)), 46);
        assert!(!cpu.status.contains(Status::ZERO));

        let mut cpu = run_instr(
            mem!(
                0 => { INC_ABS, 100, 0 }
                100 => { -1i8 as u8 }
            ),
            |_| {},
        );

        assert_eq!(cpu.read(Address::new(100)), 0);
        assert!(cpu.status.contains(Status::ZERO));
    }

    #[test]
    fn instr_inc_sets_negative_flag_based_on_result() {
        let mut cpu = run_instr(
            mem!(
                0 => { INC_ABS, 100, 0 }
                100 => { 45 }
            ),
            |_| {},
        );

        assert_eq!(cpu.read(Address::new(100)), 46);
        assert!(!cpu.status.contains(Status::ZERO));

        let mut cpu = run_instr(
            mem!(
                0 => { INC_ABS, 100, 0 }
                100 => { -10i8 as u8 }
            ),
            |_| {},
        );

        assert_eq!(cpu.read(Address::new(100)) as i8, -9i8);
        assert!(cpu.status.contains(Status::NEGATIVE));
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
    fn instr_dec_decrements_operand() {
        let mut cpu = run_instr(
            mem!(
                0 => { DEC_ABS, 100, 0 }
                100 => { 45 }
            ),
            |_| {},
        );

        assert_eq!(cpu.read(Address::new(100)), 44);
    }

    #[test]
    fn instr_dec_sets_zero_flag_based_on_result() {
        let mut cpu = run_instr(
            mem!(
                0 => { DEC_ABS, 100, 0 }
                100 => { 45 }
            ),
            |_| {},
        );

        assert_eq!(cpu.read(Address::new(100)), 44);
        assert!(!cpu.status.contains(Status::ZERO));

        let mut cpu = run_instr(
            mem!(
                0 => { DEC_ABS, 100, 0 }
                100 => { 1 }
            ),
            |_| {},
        );

        assert_eq!(cpu.read(Address::new(100)), 0);
        assert!(cpu.status.contains(Status::ZERO));
    }

    #[test]
    fn instr_dec_sets_negative_flag_based_on_result() {
        let mut cpu = run_instr(
            mem!(
                0 => { DEC_ABS, 100, 0 }
                100 => { 45 }
            ),
            |_| {},
        );

        assert_eq!(cpu.read(Address::new(100)), 44);
        assert!(!cpu.status.contains(Status::ZERO));

        let mut cpu = run_instr(
            mem!(
                0 => { DEC_ABS, 100, 0 }
                100 => { 0 }
            ),
            |_| {},
        );

        assert_eq!(cpu.read(Address::new(100)) as i8, -1i8);
        assert!(cpu.status.contains(Status::NEGATIVE));
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
        assert!(!cpu.status.contains(Status::ZERO));

        let cpu = run_instr(mem!(DEX), |cpu| {
            cpu.x = 1;
        });

        assert_eq!(cpu.x, 0);
        assert!(cpu.status.contains(Status::ZERO));
    }

    #[test]
    fn instr_dex_sets_negative_flag_based_on_result() {
        let cpu = run_instr(mem!(DEX), |cpu| {
            cpu.x = 45;
        });

        assert_eq!(cpu.x, 44);
        assert!(!cpu.status.contains(Status::ZERO));

        let cpu = run_instr(mem!(DEX), |cpu| {
            cpu.x = 0;
        });

        assert_eq!(cpu.x as i8, -1i8);
        assert!(cpu.status.contains(Status::NEGATIVE));
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
        assert!(!cpu.status.contains(Status::ZERO));

        let cpu = run_instr(mem!(DEY), |cpu| {
            cpu.y = 1;
        });

        assert_eq!(cpu.y, 0);
        assert!(cpu.status.contains(Status::ZERO));
    }

    #[test]
    fn instr_dey_sets_negative_flag_based_on_result() {
        let cpu = run_instr(mem!(DEY), |cpu| {
            cpu.y = 45;
        });

        assert_eq!(cpu.y, 44);
        assert!(!cpu.status.contains(Status::ZERO));

        let cpu = run_instr(mem!(DEY), |cpu| {
            cpu.y = 0;
        });

        assert_eq!(cpu.y as i8, -1i8);
        assert!(cpu.status.contains(Status::NEGATIVE));
    }
}
