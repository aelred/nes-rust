pub trait PPURegisters {
    fn write_control(&mut self, byte: u8);

    fn write_mask(&mut self, byte: u8);

    fn read_status(&mut self) -> u8;

    fn write_oam_address(&mut self, byte: u8);

    fn read_oam_data(&mut self) -> u8;

    fn write_oam_data(&mut self, byte: u8);

    fn write_scroll(&mut self, byte: u8);

    fn write_address(&mut self, byte: u8);

    fn read_data(&mut self) -> u8;

    fn write_data(&mut self, byte: u8);

    fn write_oam_dma(&mut self, bytes: [u8; 256]);
}

impl<'a, T: PPURegisters> PPURegisters for &'a mut T {
    fn write_control(&mut self, byte: u8) {
        (*self).write_control(byte)
    }

    fn write_mask(&mut self, byte: u8) {
        (*self).write_mask(byte)
    }

    fn read_status(&mut self) -> u8 {
        (*self).read_status()
    }

    fn write_oam_address(&mut self, byte: u8) {
        (*self).write_oam_address(byte)
    }

    fn read_oam_data(&mut self) -> u8 {
        (*self).read_oam_data()
    }

    fn write_oam_data(&mut self, byte: u8) {
        (*self).write_oam_data(byte)
    }

    fn write_scroll(&mut self, byte: u8) {
        (*self).write_scroll(byte)
    }

    fn write_address(&mut self, byte: u8) {
        (*self).write_address(byte)
    }

    fn read_data(&mut self) -> u8 {
        (*self).read_data()
    }

    fn write_data(&mut self, byte: u8) {
        (*self).write_data(byte)
    }

    fn write_oam_dma(&mut self, bytes: [u8; 256]) {
        (*self).write_oam_dma(bytes)
    }
}

