use cart::NESFile;
use mapper::{Mapper, NROMMapper};

pub mod cart;
pub mod mapper;

const MAPPER_SPACE: usize = 0x10000 - 0x4020;

pub trait Memory {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8) -> ();
}

// TODO: create a proper data types to handle generic memory for cartridges and all that good stuff
type DummyMapper = [u8; MAPPER_SPACE];

impl Memory for DummyMapper {
    fn read(&self, addr: u16) -> u8 {
        self[(addr as usize) - 0x4020]
    }

    fn write(&mut self, _addr: u16, _val: u8) -> () {
        // self[(addr as usize) - 0x4020] = val;
    }
}

// TODO: ???
impl Mapper for DummyMapper {
    fn read_chr(&self, _addr: u16) -> u8 {
        0xff
    }

    fn write_chr(&mut self, _addr: u16, _val: u8) -> () {
        ()
    }
}

// TODO: Temp. In this case, PRG == CHR
impl Memory for Vec<u8> {
    fn read(&self, addr: u16) -> u8 {
        self[addr as usize]
    }

    fn write(&mut self, addr: u16, val: u8) -> () {
        self[addr as usize] = val;
    }
}

impl Mapper for Vec<u8> {
    fn read_chr(&self, addr: u16) -> u8 {
        self[addr as usize]
    }

    fn write_chr(&mut self, addr: u16, val: u8) -> () {
        self[addr as usize] = val;
    }
}

pub struct MemMap {
    ram: [u8; 0x800],
    // ...  // TODO: ppu, apu, ...
    pub mapper: Box<dyn Mapper>,
}

impl Memory for MemMap {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x07ff => self.ram[addr as usize],
            0x0800..=0x1fff => self.ram[(addr % 0x800) as usize],
            0x4000..=0x4017 => 0xff, // TODO: apu registers
            0x4018..=0x401f => 0xff, // TODO: apu test mode things?
            0x4020..=0xffff => self.mapper.read(addr),
            _ => unreachable!("Attempted to read PPU MMIO (address ${:04x})", addr),
        }
    }

    fn write(&mut self, addr: u16, val: u8) -> () {
        match addr {
            0x0000..=0x07ff => self.ram[addr as usize] = val,
            0x0800..=0x1fff => self.ram[(addr % 0x800) as usize] = val,
            0x4000..=0x4017 => (), // TODO: apu registers
            0x4018..=0x401f => (), // TODO: apu test mode things?
            0x4020..=0xffff => self.mapper.write(addr, val),
            _ => unreachable!("Attempted to read PPU MMIO (address ${:04x})", addr),
        };
    }
}

impl MemMap {
    pub fn empty() -> MemMap {
        MemMap {
            ram: [0; 0x800],
            mapper: Box::new([0; MAPPER_SPACE]),
        }
    }

    pub fn from_nesfile(nesfile: &NESFile) -> MemMap {
        match nesfile.mapper_type() {
            mapper::MapperType::NROM => {
                let mapper = Box::new(NROMMapper::from_nesfile(nesfile));
                MemMap {
                    ram: [0; 0x800],
                    mapper,
                }
            }
            mapper::MapperType::UNKNOWN(i) => {
                println!("WARNING: UNKNOWN MAPPER ({i})");
                println!("as a fallback, RAM is used as a mapper (r/w)");

                let buf = nesfile.data.clone();

                let mut data = Box::new([0; MAPPER_SPACE]);
                for i in 0..buf.len() {
                    data[i] = buf[i];
                }
                MemMap {
                    ram: [0; 0x800],
                    mapper: data,
                }
            }
        }
    }
}
