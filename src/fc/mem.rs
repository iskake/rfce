use std::io;
use std::io::prelude::*;
use std::fs::File;

pub trait Memory {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8) -> ();
}

// TODO: create a proper data type to handle generic memory for cartridges and all that good stuff
pub struct FCMem {
    data: [u8; 0x10000],
}

impl Memory for FCMem {
    fn read(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    fn write(&mut self, addr: u16, val: u8) -> () {
        self.data[addr as usize] = val;
    }
}

impl FCMem {
    pub fn empty() -> FCMem {
        FCMem { data: [0; 0x10000] }
    }

    pub fn from_file(filename: &str) -> Result<FCMem, io::Error> {
        // TODO: fix this mess
        let mut f = File::open(filename)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        
        let mut data = [0; 0x10000];
        for i in 0..buf.len() {
            data[0x8000+i] = buf[i];
        }
        // let data: [u8; 0x8000] = buf.try_into()
        //     .unwrap_or_else(|v: Vec<u8>| panic!("Length of file is invalid: {} (expected {})", v.len(), 0x8000));
        Ok(FCMem { data })
    }
}