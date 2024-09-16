//! Arithmetic operations

use crate::{
    cpu::addressing_modes::{CompareAddressingMode, FlexibleAddressingMode},
    Memory, CPU,
};

impl<M: Memory> CPU<M> {
    pub(in crate::cpu) fn adc(&mut self, addressing_mode: FlexibleAddressingMode) {
        let value = self.fetch(addressing_mode);
        self.add_to_accumulator(value);
    }

    pub(in crate::cpu) fn sbc(&mut self, addressing_mode: FlexibleAddressingMode) {
        let value = self.fetch(addressing_mode);
        self.sub_from_accumulator(value);
    }

    pub(in crate::cpu) fn cmp(&mut self, addressing_mode: FlexibleAddressingMode) {
        let value = self.fetch(addressing_mode);
        self.compare(self.accumulator, value);
    }

    pub(in crate::cpu) fn cpx(&mut self, addressing_mode: CompareAddressingMode) {
        let value = self.fetch(addressing_mode);
        self.compare(self.x, value);
    }

    pub(in crate::cpu) fn cpy(&mut self, addressing_mode: CompareAddressingMode) {
        let value = self.fetch(addressing_mode);
        self.compare(self.y, value);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::{tests::run_instr, Status},
        instructions::{ADC_IMM, CMP_IMM, CPX_IMM, CPY_IMM, SBC_IMM},
        mem,
    };

    #[test]
    fn instr_adc_adds_numbers() {
        let cpu = run_instr(mem!(ADC_IMM, 10u8), |cpu| {
            cpu.accumulator = 42;
        });

        assert_eq!(cpu.accumulator, 52);
        assert!(!cpu.status.contains(Status::OVERFLOW));
        assert!(!cpu.status.contains(Status::CARRY));
    }

    #[test]
    fn instr_adc_sets_carry_flag_on_unsigned_overflow() {
        let cpu = run_instr(mem!(ADC_IMM, 255u8), |cpu| {
            cpu.accumulator = 42;
        });

        assert_eq!(cpu.accumulator, 41);
        assert!(!cpu.status.contains(Status::OVERFLOW));
        assert!(cpu.status.contains(Status::CARRY));
    }

    #[test]
    fn instr_adc_sets_overflow_flag_on_signed_overflow() {
        let cpu = run_instr(mem!(ADC_IMM, 127u8), |cpu| {
            cpu.accumulator = 42i8 as u8;
        });

        assert_eq!(cpu.accumulator as i8, -87i8);
        assert!(cpu.status.contains(Status::OVERFLOW));
        assert!(!cpu.status.contains(Status::CARRY));
    }

    #[test]
    fn instr_sbc_subtracts_numbers() {
        let cpu = run_instr(mem!(SBC_IMM, 10u8), |cpu| {
            cpu.status.insert(Status::CARRY);
            cpu.accumulator = 42;
        });

        assert_eq!(cpu.accumulator, 32);
        assert!(cpu.status.contains(Status::CARRY));
    }

    #[test]
    fn instr_sbc_sets_overflow_bit_when_sign_is_wrong() {
        fn sub(accumulator: i8, value: i8) -> (i8, bool) {
            let cpu = run_instr(mem!(SBC_IMM, value as u8), |cpu| {
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
        let cpu = run_instr(mem!(CMP_IMM, 10u8), |cpu| {
            cpu.accumulator = 1;
        });

        assert!(!cpu.status.contains(Status::CARRY));

        let cpu = run_instr(mem!(CMP_IMM, 10u8), |cpu| {
            cpu.accumulator = 10;
        });

        assert!(cpu.status.contains(Status::CARRY));

        let cpu = run_instr(mem!(CMP_IMM, 10u8), |cpu| {
            cpu.accumulator = 100;
        });

        assert!(cpu.status.contains(Status::CARRY));
    }

    #[test]
    fn instr_cmp_sets_zero_flag_if_accumulator_equals_operand() {
        let cpu = run_instr(mem!(CMP_IMM, 10u8), |cpu| {
            cpu.accumulator = 1;
        });

        assert!(!cpu.status.contains(Status::ZERO));

        let cpu = run_instr(mem!(CMP_IMM, 10u8), |cpu| {
            cpu.accumulator = 10;
        });

        assert!(cpu.status.contains(Status::ZERO));

        let cpu = run_instr(mem!(CMP_IMM, 10u8), |cpu| {
            cpu.accumulator = 100;
        });

        assert!(!cpu.status.contains(Status::ZERO));
    }

    #[test]
    fn instr_cmp_sets_negative_flag_if_bit_7_of_accumulator_sub_operand_is_set() {
        let cpu = run_instr(mem!(CMP_IMM, 10u8), |cpu| {
            cpu.accumulator = 1;
        });

        assert!(cpu.status.contains(Status::NEGATIVE));

        let cpu = run_instr(mem!(CMP_IMM, 10u8), |cpu| {
            cpu.accumulator = 10;
        });

        assert!(!cpu.status.contains(Status::NEGATIVE));

        let cpu = run_instr(mem!(CMP_IMM, 10u8), |cpu| {
            cpu.accumulator = 100;
        });

        assert!(!cpu.status.contains(Status::NEGATIVE));
    }

    #[test]
    fn instr_cpx_compares_using_x_register() {
        let cpu = run_instr(mem!(CPX_IMM, 10u8), |cpu| {
            cpu.x = 1;
        });

        assert!(!cpu.status.contains(Status::CARRY));
        assert!(!cpu.status.contains(Status::ZERO));
        assert!(cpu.status.contains(Status::NEGATIVE));

        let cpu = run_instr(mem!(CPX_IMM, 10u8), |cpu| {
            cpu.x = 10;
        });

        assert!(cpu.status.contains(Status::CARRY));
        assert!(cpu.status.contains(Status::ZERO));
        assert!(!cpu.status.contains(Status::NEGATIVE));

        let cpu = run_instr(mem!(CPX_IMM, 10u8), |cpu| {
            cpu.x = 100;
        });

        assert!(cpu.status.contains(Status::CARRY));
        assert!(!cpu.status.contains(Status::ZERO));
        assert!(!cpu.status.contains(Status::NEGATIVE));
    }

    #[test]
    fn instr_cpy_compares_using_y_register() {
        let cpu = run_instr(mem!(CPY_IMM, 10u8), |cpu| {
            cpu.y = 1;
        });

        assert!(!cpu.status.contains(Status::CARRY));
        assert!(!cpu.status.contains(Status::ZERO));
        assert!(cpu.status.contains(Status::NEGATIVE));

        let cpu = run_instr(mem!(CPY_IMM, 10u8), |cpu| {
            cpu.y = 10;
        });

        assert!(cpu.status.contains(Status::CARRY));
        assert!(cpu.status.contains(Status::ZERO));
        assert!(!cpu.status.contains(Status::NEGATIVE));

        let cpu = run_instr(mem!(CPY_IMM, 10u8), |cpu| {
            cpu.y = 100;
        });

        assert!(cpu.status.contains(Status::CARRY));
        assert!(!cpu.status.contains(Status::ZERO));
        assert!(!cpu.status.contains(Status::NEGATIVE));
    }

    #[test]
    fn addition_behaves_appropriately_across_many_values() {
        let carry_values = [true, false];
        let values = [0, 1, 2, 3, 126, 127, 128, 129, 252, 253, 254, 255];

        for x in values.iter() {
            for y in values.iter() {
                for carry_in in carry_values.iter() {
                    let cpu = run_instr(mem!(ADC_IMM, *y), |cpu| {
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
                    let cpu = run_instr(mem!(SBC_IMM, *y), |cpu| {
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
}
