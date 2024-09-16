//! Branches

use crate::{cpu::Status, Memory, CPU};

impl<M: Memory> CPU<M> {
    pub(in crate::cpu) fn bcc(&mut self) {
        self.branch_if(!self.status.contains(Status::CARRY))
    }

    pub(in crate::cpu) fn bcs(&mut self) {
        self.branch_if(self.status.contains(Status::CARRY))
    }

    pub(in crate::cpu) fn beq(&mut self) {
        self.branch_if(self.status.contains(Status::ZERO))
    }

    pub(in crate::cpu) fn bmi(&mut self) {
        self.branch_if(self.status.contains(Status::NEGATIVE))
    }

    pub(in crate::cpu) fn bne(&mut self) {
        self.branch_if(!self.status.contains(Status::ZERO))
    }

    pub(in crate::cpu) fn bpl(&mut self) {
        self.branch_if(!self.status.contains(Status::NEGATIVE))
    }

    pub(in crate::cpu) fn bvc(&mut self) {
        self.branch_if(!self.status.contains(Status::OVERFLOW))
    }

    pub(in crate::cpu) fn bvs(&mut self) {
        self.branch_if(self.status.contains(Status::OVERFLOW))
    }

    fn branch_if(&mut self, cond: bool) {
        let offset = self.incr_program_counter() as i8;
        if cond {
            let previous = self.program_counter;
            self.program_counter += offset as u16;
            self.cycle_count += 1;
            if self.program_counter.page_crossed(previous) {
                self.cycle_count += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::{tests::run_instr, Status},
        instructions::{BCC, BCS, BEQ, BMI, BNE, BPL, BVC, BVS},
        mem, Address,
    };

    #[test]
    fn instr_bcc_branches_when_carry_flag_clear() {
        let cpu = run_instr(mem!(90 => { BCC, -10i8 as u8 }), |cpu| {
            cpu.program_counter = Address::new(90);
            cpu.status.remove(Status::CARRY);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter, Address::new(82));
    }

    #[test]
    fn instr_bcc_does_not_branch_when_carry_flag_set() {
        let cpu = run_instr(mem!(90 => { BCC, -10i8 as u8 }), |cpu| {
            cpu.program_counter = Address::new(90);
            cpu.status.insert(Status::CARRY);
        });

        assert_eq!(cpu.program_counter, Address::new(92));
    }

    #[test]
    fn instr_bcs_does_not_branch_when_carry_flag_clear() {
        let cpu = run_instr(mem!(90 => { BCS, -10i8 as u8 }), |cpu| {
            cpu.program_counter = Address::new(90);
            cpu.status.remove(Status::CARRY);
        });

        assert_eq!(cpu.program_counter, Address::new(92));
    }

    #[test]
    fn instr_bcs_branches_when_carry_flag_set() {
        let cpu = run_instr(mem!(90 => { BCS, -10i8 as u8 }), |cpu| {
            cpu.program_counter = Address::new(90);
            cpu.status.insert(Status::CARRY);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter, Address::new(82));
    }

    #[test]
    fn instr_beq_does_not_branch_when_zero_flag_clear() {
        let cpu = run_instr(mem!(90 => { BEQ, -10i8 as u8 }), |cpu| {
            cpu.program_counter = Address::new(90);
            cpu.status.remove(Status::ZERO);
        });

        assert_eq!(cpu.program_counter, Address::new(92));
    }

    #[test]
    fn instr_beq_branches_when_zero_flag_set() {
        let cpu = run_instr(mem!(90 => { BEQ, -10i8 as u8 }), |cpu| {
            cpu.program_counter = Address::new(90);
            cpu.status.insert(Status::ZERO);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter, Address::new(82));
    }

    #[test]
    fn instr_bmi_does_not_branch_when_negative_flag_clear() {
        let cpu = run_instr(mem!(90 => { BMI, -10i8 as u8 }), |cpu| {
            cpu.program_counter = Address::new(90);
            cpu.status.remove(Status::NEGATIVE);
        });

        assert_eq!(cpu.program_counter, Address::new(92));
    }

    #[test]
    fn instr_bmi_branches_when_negative_flag_set() {
        let cpu = run_instr(mem!(90 => { BMI, -10i8 as u8 }), |cpu| {
            cpu.program_counter = Address::new(90);
            cpu.status.insert(Status::NEGATIVE);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter, Address::new(82));
    }

    #[test]
    fn instr_bne_branches_when_zero_flag_clear() {
        let cpu = run_instr(mem!(90 => { BNE, -10i8 as u8 }), |cpu| {
            cpu.program_counter = Address::new(90);
            cpu.status.remove(Status::ZERO);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter, Address::new(82));
    }

    #[test]
    fn instr_bne_does_not_branch_when_zero_flag_set() {
        let cpu = run_instr(mem!(90 => { BNE, -10i8 as u8 }), |cpu| {
            cpu.program_counter = Address::new(90);
            cpu.status.insert(Status::ZERO);
        });

        assert_eq!(cpu.program_counter, Address::new(92));
    }

    #[test]
    fn instr_bpl_branches_when_negative_flag_clear() {
        let cpu = run_instr(mem!(90 => { BPL, -10i8 as u8 }), |cpu| {
            cpu.program_counter = Address::new(90);
            cpu.status.remove(Status::NEGATIVE);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter, Address::new(82));
    }

    #[test]
    fn instr_bpl_does_not_branch_when_negative_flag_set() {
        let cpu = run_instr(mem!(90 => { BPL, -10i8 as u8 }), |cpu| {
            cpu.program_counter = Address::new(90);
            cpu.status.insert(Status::NEGATIVE);
        });

        assert_eq!(cpu.program_counter, Address::new(92));
    }

    #[test]
    fn instr_bvc_branches_when_overflow_flag_clear() {
        let cpu = run_instr(mem!(90 => { BVC, -10i8 as u8 }), |cpu| {
            cpu.program_counter = Address::new(90);
            cpu.status.remove(Status::OVERFLOW);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter, Address::new(82));
    }

    #[test]
    fn instr_bvc_does_not_branch_when_overflow_flag_set() {
        let cpu = run_instr(mem!(90 => { BVC, -10i8 as u8 }), |cpu| {
            cpu.program_counter = Address::new(90);
            cpu.status.insert(Status::OVERFLOW);
        });

        assert_eq!(cpu.program_counter, Address::new(92));
    }

    #[test]
    fn instr_bvs_does_not_branch_when_carry_flag_clear() {
        let cpu = run_instr(mem!(90 => { BVS, -10i8 as u8 }), |cpu| {
            cpu.program_counter = Address::new(90);
            cpu.status.remove(Status::OVERFLOW);
        });

        assert_eq!(cpu.program_counter, Address::new(92));
    }

    #[test]
    fn instr_bvs_branches_when_carry_flag_set() {
        let cpu = run_instr(mem!(90 => { BVS, -10i8 as u8 }), |cpu| {
            cpu.program_counter = Address::new(90);
            cpu.status.insert(Status::OVERFLOW);
        });

        // 2 steps ahead because PC also automatically increments
        assert_eq!(cpu.program_counter, Address::new(82));
    }
}
