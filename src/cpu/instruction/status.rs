//! Status Flag Changes

use crate::{cpu::Status, Memory, CPU};

impl<M: Memory> CPU<M> {
    pub(in crate::cpu) fn clc(&mut self) {
        self.ignore_argument();
        self.status.remove(Status::CARRY);
    }

    pub(in crate::cpu) fn cld(&mut self) {
        self.ignore_argument();
        self.status.remove(Status::DECIMAL);
    }

    pub(in crate::cpu) fn cli(&mut self) {
        self.ignore_argument();
        self.status.remove(Status::INTERRUPT_DISABLE);
    }

    pub(in crate::cpu) fn clv(&mut self) {
        self.ignore_argument();
        self.status.remove(Status::OVERFLOW);
    }

    pub(in crate::cpu) fn sec(&mut self) {
        self.ignore_argument();
        self.status.insert(Status::CARRY);
    }

    pub(in crate::cpu) fn sed(&mut self) {
        self.ignore_argument();
        self.status.insert(Status::DECIMAL);
    }

    pub(in crate::cpu) fn sei(&mut self) {
        self.ignore_argument();
        self.status.insert(Status::INTERRUPT_DISABLE);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::{tests::run_instr, Status},
        instructions::{CLC, CLD, CLI, CLV, SEC, SED, SEI},
        mem,
    };

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
}
