use cart::NESFile;
use mapper::NROMMapper;

pub mod cart;
mod mapper;

const MAPPER_SPACE: usize = 0x10000 - 0x4020;

pub trait Memory {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8) -> ();
}

// TODO: create a proper data types to handle generic memory for cartridges and all that good stuff
type SimpleMapper = [u8; MAPPER_SPACE];

impl Memory for SimpleMapper {
    fn read(&self, addr: u16) -> u8 {
        self[(addr as usize) - 0x4020]
    }

    fn write(&mut self, _addr: u16, _val: u8) -> () {
        // self[(addr as usize) - 0x4020] = val;
    }
}

pub struct MemMap {
    ram: [u8; 0x800],
    // ...  // TODO: ppu, apu, ...
    mapper: Box<dyn Memory>,
}

impl Memory for MemMap {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x07ff => self.ram[addr as usize],
            0x0800..=0x1fff => self.ram[(addr % 0x800) as usize],
            0x2000..=0x2007 => 0xff,                   // TODO: ppu registers
            0x2008..=0x3fff => 0xff,                   // TODO: mirrors of ppu registers
            0x4000..=0x4017 => 0xff,                   // TODO: apu registers
            0x4018..=0x401f => 0xff,                   // TODO: apu test mode things
            0x4020..=0xffff => self.mapper.read(addr), // TODO: real
        }
    }

    fn write(&mut self, addr: u16, val: u8) -> () {
        match addr {
            0x0000..=0x07ff => self.ram[addr as usize] = val,
            0x0800..=0x1fff => self.ram[(addr % 0x800) as usize] = val,
            0x2000..=0x2007 => (), // TODO: ppu registers
            0x2008..=0x3fff => (), // TODO: mirrors of ppu registers
            0x4000..=0x4017 => (), // TODO: apu registers
            0x4018..=0x401f => (), // TODO: apu test mode things
            0x4020..=0xffff => self.mapper.write(addr, val), // TODO: real
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

    pub fn from_nesfile(nesfile: NESFile) -> MemMap {
        println!("{:?}", nesfile.mapper_type());
        match nesfile.mapper_type() {
            mapper::MapperType::NROM => {
                let mapper = Box::new(NROMMapper::from_nesfile(nesfile));
                MemMap { ram: [0; 0x800], mapper }
            },
            mapper::MapperType::UNKNOWN(i) => {
                println!("WARNING: UNKNOWN MAPPER ({i})");
                println!("as a fallback, RAM is used as a mapper (r/w)");

                let buf = nesfile.data;

                let mut data = Box::new([0; MAPPER_SPACE]);
                for i in 0..buf.len() {
                    data[i] = buf[i];
                }
                // let data: [u8; 0x8000] = buf.try_into()
                //     .unwrap_or_else(|v: Vec<u8>| panic!("Length of file is invalid: {} (expected {})", v.len(), 0x8000));
                MemMap {
                    ram: [0; 0x800],
                    mapper: data,
                }
            },
        }
    }
}
