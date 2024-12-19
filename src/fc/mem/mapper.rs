#[derive(Debug)]
pub enum MapperType {
    NROM,
    UNKNOWN(u16),
}

pub struct NROMMapper {

}