//! Load/Store Operations
use crate::{
    cpu::addressing_modes::{
        FlexibleAddressingMode, LAXAddressingMode, LDXAddressingMode, LDYAddressingMode,
        SAXAddressingMode, STXAddressingMode, STYAddressingMode, StoreAddressingMode,
    },
    Memory, CPU,
};

impl<M: Memory> CPU<M> {
    pub fn lda(&mut self, addressing_mode: FlexibleAddressingMode) {
        let value = self.fetch(addressing_mode);
        self.set_accumulator(value);
    }

    pub fn ldx(&mut self, addressing_mode: LDXAddressingMode) {
        let value = self.fetch(addressing_mode);
        self.set_x(value);
    }

    pub fn ldy(&mut self, addressing_mode: LDYAddressingMode) {
        let value = self.fetch(addressing_mode);
        self.set_y(value);
    }

    pub fn sta(&mut self, addressing_mode: StoreAddressingMode) {
        let reference = self.fetch_ref(addressing_mode);
        self.write_reference(reference, self.accumulator, true);
    }

    pub fn stx(&mut self, addressing_mode: STXAddressingMode) {
        let reference = self.fetch_ref(addressing_mode);
        self.write_reference(reference, self.x, true);
    }

    pub fn sty(&mut self, addressing_mode: STYAddressingMode) {
        let reference = self.fetch_ref(addressing_mode);
        self.write_reference(reference, self.y, true);
    }

    // Unofficial Opcodes
    pub fn lax(&mut self, addressing_mode: LAXAddressingMode) {
        let value = self.fetch(addressing_mode);
        self.set_accumulator(value);
        self.set_x(value);
    }

    pub fn sax(&mut self, addressing_mode: SAXAddressingMode) {
        let reference = self.fetch_ref(addressing_mode);
        self.write_reference(reference, self.accumulator & self.x, true);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cpu::tests::run_instr,
        instructions::{LDA_IMM, LDX_IMM, LDY_IMM, STA_ABS, STX_ABS, STY_ABS},
        mem, Address,
    };

    #[test]
    fn instr_lda_loads_operand_into_accunmulator() {
        let cpu = run_instr(mem!(LDA_IMM, 5u8), |_| {});

        assert_eq!(cpu.accumulator, 5);
    }

    #[test]
    fn instr_ldx_loads_operand_into_x_register() {
        let cpu = run_instr(mem!(LDX_IMM, 5u8), |_| {});

        assert_eq!(cpu.x, 5);
    }

    #[test]
    fn instr_ldy_loads_operand_into_y_register() {
        let cpu = run_instr(mem!(LDY_IMM, 5u8), |_| {});

        assert_eq!(cpu.y, 5);
    }

    #[test]
    fn instr_sta_stores_accumulator_in_memory() {
        let mut cpu = run_instr(mem!(STA_ABS, 0x32, 0), |cpu| {
            cpu.accumulator = 65;
        });

        assert_eq!(cpu.read(Address::new(0x32)), 65);
    }

    #[test]
    fn instr_stx_stores_x_register_in_memory() {
        let mut cpu = run_instr(mem!(STX_ABS, 0x32, 0), |cpu| {
            cpu.x = 65;
        });

        assert_eq!(cpu.read(Address::new(0x32)), 65);
    }

    #[test]
    fn instr_sty_stores_y_register_in_memory() {
        let mut cpu = run_instr(mem!(STY_ABS, 0x32, 0), |cpu| {
            cpu.y = 65;
        });

        assert_eq!(cpu.read(Address::new(0x32)), 65);
    }
}
