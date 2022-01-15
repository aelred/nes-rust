#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
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