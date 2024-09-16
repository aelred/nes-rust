//! Register Transfers

use crate::{Memory, CPU};

impl<M: Memory> CPU<M> {
    pub(in crate::cpu) fn tax(&mut self) {
        self.ignore_argument();
        self.set_x(self.accumulator);
    }

    pub(in crate::cpu) fn tay(&mut self) {
        self.ignore_argument();
        self.set_y(self.accumulator);
    }

    pub(in crate::cpu) fn txa(&mut self) {
        self.ignore_argument();
        self.set_accumulator(self.x);
    }

    pub(in crate::cpu) fn tya(&mut self) {
        self.ignore_argument();
        self.set_accumulator(self.y);
    }

    pub(in crate::cpu) fn tsx(&mut self) {
        self.ignore_argument();
        self.set_x(self.stack_pointer.0);
    }

    pub(in crate::cpu) fn txs(&mut self) {
        self.ignore_argument();
        self.stack_pointer.0 = self.x;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::{tests::run_instr, Status},
        instructions::{TAX, TAY, TSX, TXA, TXS, TYA},
        mem,
    };

    #[test]
    fn instr_tax_transfers_accumulator_to_x_register() {
        let cpu = run_instr(mem!(TAX), |cpu| {
            cpu.accumulator = 65;
        });

        assert_eq!(cpu.x, 65);
    }

    #[test]
    fn instr_tay_transfers_accumulator_to_y_register() {
        let cpu = run_instr(mem!(TAY), |cpu| {
            cpu.accumulator = 65;
        });

        assert_eq!(cpu.y, 65);
    }

    #[test]
    fn instr_txa_transfers_x_register_to_accumulator() {
        let cpu = run_instr(mem!(TXA), |cpu| {
            cpu.x = 65;
        });

        assert_eq!(cpu.accumulator, 65);
    }

    #[test]
    fn instr_tya_transfers_y_register_to_accumulator() {
        let cpu = run_instr(mem!(TYA), |cpu| {
            cpu.y = 65;
        });

        assert_eq!(cpu.accumulator, 65);
    }

    #[test]
    fn instr_tsx_transfers_stack_pointer_to_x_register() {
        let cpu = run_instr(mem!(TSX), |cpu| {
            cpu.stack_pointer.0 = 65;
        });

        assert_eq!(cpu.x, 65);
    }

    #[test]
    fn instr_txs_transfers_x_register_to_stack_pointer() {
        let cpu = run_instr(mem!(TXS), |cpu| {
            cpu.x = 65;
        });

        assert_eq!(cpu.stack_pointer.0, 65);
    }

    #[test]
    fn instr_txs_does_not_modify_zero_or_negative_register() {
        let cpu = run_instr(mem!(TXS), |cpu| {
            cpu.x = 65;
            cpu.status.insert(Status::ZERO);
            cpu.status.insert(Status::NEGATIVE);
        });

        assert!(cpu.status.contains(Status::ZERO));
        assert!(cpu.status.contains(Status::NEGATIVE));
    }
}
