//! Arithmetic operations

use crate::{
    cpu::addressing_modes::{CompareAddressingMode, FlexibleAddressingMode},
    Memory, CPU,
};

impl<M: Memory> CPU<M> {
    pub fn adc(&mut self, addressing_mode: FlexibleAddressingMode) {
        let value = self.fetch(addressing_mode);
        self.add_to_accumulator(value);
    }

    pub fn sbc(&mut self, addressing_mode: FlexibleAddressingMode) {
        let value = self.fetch(addressing_mode);
        self.sub_from_accumulator(value);
    }

    pub fn cmp(&mut self, addressing_mode: FlexibleAddressingMode) {
        let value = self.fetch(addressing_mode);
        self.compare(self.accumulator, value);
    }

    pub fn cpx(&mut self, addressing_mode: CompareAddressingMode) {
        let value = self.fetch(addressing_mode);
        self.compare(self.x, value);
    }

    pub fn cpy(&mut self, addressing_mode: CompareAddressingMode) {
        let value = self.fetch(addressing_mode);
        self.compare(self.y, value);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::{tests::run_instr, Status},
        instructions::{ADC_IMMEDIATE, CMP_IMMEDIATE, CPX_IMMEDIATE, CPY_IMMEDIATE, SBC_IMMEDIATE},
        mem,
    };

    #[test]
    fn instr_adc_adds_numbers() {
        let cpu = run_instr(mem!(ADC_IMMEDIATE, 10u8), |cpu| {
            cpu.accumulator = 42;
        });

        assert_eq!(cpu.accumulator, 52);
        assert!(!cpu.status.contains(Status::OVERFLOW));
        assert!(!cpu.status.contains(Status::CARRY));
    }

    #[test]
    fn instr_adc_sets_carry_flag_on_unsigned_overflow() {
        let cpu = run_instr(mem!(ADC_IMMEDIATE, 255u8), |cpu| {
            cpu.accumulator = 42;
        });

        assert_eq!(cpu.accumulator, 41);
        assert!(!cpu.status.contains(Status::OVERFLOW));
        assert!(cpu.status.contains(Status::CARRY));
    }

    #[test]
    fn instr_adc_sets_overflow_flag_on_signed_overflow() {
        let cpu = run_instr(mem!(ADC_IMMEDIATE, 127u8), |cpu| {
            cpu.accumulator = 42i8 as u8;
        });

        assert_eq!(cpu.accumulator as i8, -87i8);
        assert!(cpu.status.contains(Status::OVERFLOW));
        assert!(!cpu.status.contains(Status::CARRY));
    }

    #[test]
    fn instr_sbc_subtracts_numbers() {
        let cpu = run_instr(mem!(SBC_IMMEDIATE, 10u8), |cpu| {
            cpu.status.insert(Status::CARRY);
            cpu.accumulator = 42;
        });

        assert_eq!(cpu.accumulator, 32);
        assert!(cpu.status.contains(Status::CARRY));
    }

    #[test]
    fn instr_sbc_sets_overflow_bit_when_sign_is_wrong() {
        fn sub(accumulator: i8, value: i8) -> (i8, bool) {
            let cpu = run_instr(mem!(SBC_IMMEDIATE, value as u8), |cpu| {
                cpu.status.insert(Status::CARRY);
                cpu.accumulator = accumulator as u8;
            });

            (
                cpu.accumulator as i8,
                {
                    let this = &cpu;
                    this.status
                }
                .contains(Status::OVERFLOW),
            )
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
    fn instr_cmp_sets_carry_flag_if_accumulator_greater_or_equal_to_operand() {
        let cpu = run_instr(mem!(CMP_IMMEDIATE, 10u8), |cpu| {
            cpu.accumulator = 1;
        });

        assert!(!cpu.status.contains(Status::CARRY));

        let cpu = run_instr(mem!(CMP_IMMEDIATE, 10u8), |cpu| {
            cpu.accumulator = 10;
        });

        assert!(cpu.status.contains(Status::CARRY));

        let cpu = run_instr(mem!(CMP_IMMEDIATE, 10u8), |cpu| {
            cpu.accumulator = 100;
        });

        assert!(cpu.status.contains(Status::CARRY));
    }

    #[test]
    fn instr_cmp_sets_zero_flag_if_accumulator_equals_operand() {
        let cpu = run_instr(mem!(CMP_IMMEDIATE, 10u8), |cpu| {
            cpu.accumulator = 1;
        });

        assert!(!cpu.status.contains(Status::ZERO));

        let cpu = run_instr(mem!(CMP_IMMEDIATE, 10u8), |cpu| {
            cpu.accumulator = 10;
        });

        assert!(cpu.status.contains(Status::ZERO));

        let cpu = run_instr(mem!(CMP_IMMEDIATE, 10u8), |cpu| {
            cpu.accumulator = 100;
        });

        assert!(!cpu.status.contains(Status::ZERO));
    }

    #[test]
    fn instr_cmp_sets_negative_flag_if_bit_7_of_accumulator_sub_operand_is_set() {
        let cpu = run_instr(mem!(CMP_IMMEDIATE, 10u8), |cpu| {
            cpu.accumulator = 1;
        });

        assert!(cpu.status.contains(Status::NEGATIVE));

        let cpu = run_instr(mem!(CMP_IMMEDIATE, 10u8), |cpu| {
            cpu.accumulator = 10;
        });

        assert!(!cpu.status.contains(Status::NEGATIVE));

        let cpu = run_instr(mem!(CMP_IMMEDIATE, 10u8), |cpu| {
            cpu.accumulator = 100;
        });

        assert!(!cpu.status.contains(Status::NEGATIVE));
    }

    #[test]
    fn instr_cpx_compares_using_x_register() {
        let cpu = run_instr(mem!(CPX_IMMEDIATE, 10u8), |cpu| {
            cpu.x = 1;
        });

        assert!(!cpu.status.contains(Status::CARRY));
        assert!(!cpu.status.contains(Status::ZERO));
        assert!(cpu.status.contains(Status::NEGATIVE));

        let cpu = run_instr(mem!(CPX_IMMEDIATE, 10u8), |cpu| {
            cpu.x = 10;
        });

        assert!(cpu.status.contains(Status::CARRY));
        assert!(cpu.status.contains(Status::ZERO));
        assert!(!cpu.status.contains(Status::NEGATIVE));

        let cpu = run_instr(mem!(CPX_IMMEDIATE, 10u8), |cpu| {
            cpu.x = 100;
        });

        assert!(cpu.status.contains(Status::CARRY));
        assert!(!cpu.status.contains(Status::ZERO));
        assert!(!cpu.status.contains(Status::NEGATIVE));
    }

    #[test]
    fn instr_cpy_compares_using_y_register() {
        let cpu = run_instr(mem!(CPY_IMMEDIATE, 10u8), |cpu| {
            cpu.y = 1;
        });

        assert!(!cpu.status.contains(Status::CARRY));
        assert!(!cpu.status.contains(Status::ZERO));
        assert!(cpu.status.contains(Status::NEGATIVE));

        let cpu = run_instr(mem!(CPY_IMMEDIATE, 10u8), |cpu| {
            cpu.y = 10;
        });

        assert!(cpu.status.contains(Status::CARRY));
        assert!(cpu.status.contains(Status::ZERO));
        assert!(!cpu.status.contains(Status::NEGATIVE));

        let cpu = run_instr(mem!(CPY_IMMEDIATE, 10u8), |cpu| {
            cpu.y = 100;
        });

        assert!(cpu.status.contains(Status::CARRY));
        assert!(!cpu.status.contains(Status::ZERO));
        assert!(!cpu.status.contains(Status::NEGATIVE));
    }
}