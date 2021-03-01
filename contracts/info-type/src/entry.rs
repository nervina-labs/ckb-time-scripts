use core::result::Result;

use ckb_std::{ckb_constants::Source, high_level::load_cell_data};

use crate::error::Error;

const TIMESTAMP_DATA_LEN: usize = 5;
const BLOCK_NUMBER_DATA_LEN: usize = 9;

pub fn main() -> Result<(), Error> {
    let data = load_cell_data(0, Source::GroupOutput)?;
    if data.len() != TIMESTAMP_DATA_LEN || data.len() != BLOCK_NUMBER_DATA_LEN {
        return Err(Error::TimeInfoDataLenError);
    }

    Ok(())
}
