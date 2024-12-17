pub trait Memory {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8) -> ();
}

// TODO: create a proper data type to handle generic memory for cartridges and all that good stuff
pub type Memory_ = [u8; 0x10000];

impl Memory for Memory_ {
    fn read(&self, addr: u16) -> u8 {
        self[addr as usize]
    }

    fn write(&mut self, addr: u16, val: u8) -> () {
        self[addr as usize] = val;
    }
}