use core::result::Result;

use alloc::vec::Vec;

use ckb_std::{ckb_constants::Source, high_level::load_cell_data};

use crate::error::Error;

const SUM_OF_TIME_INFO_CELL: u8 = 12;

pub fn main() -> Result<(), Error> {
    let output_index_data = load_cell_data(0, Source::GroupOutput)?;
    let _ = check_index_state_cell_data(&output_index_data);

    load_cell_data(0, Source::GroupInput).map_or_else(
        |_e| Ok(()),
        |input_index_data| {
            let _ = check_index_state_cell_data(&input_index_data);
            if input_index_data[0] + 1 != output_index_data[0] {
                return Err(Error::TimeIndexIncreaseError);
            }
            Ok(())
        },
    )
}

// Time index state cell data: index(u8) | sum_of_time_info_cells(u8)
const INFO_STATE_CELL_DATA_LEN: usize = 2;
fn check_index_state_cell_data(data: &Vec<u8>) -> Result<(), Error> {
    if data.len() != INFO_STATE_CELL_DATA_LEN {
        return Err(Error::IndexStateDataLenError);
    }

    if data[0] >= SUM_OF_TIME_INFO_CELL {
        return Err(Error::TimeIndexOutOfBound);
    }

    if data[1] != SUM_OF_TIME_INFO_CELL {
        return Err(Error::TimeInfoAccountError);
    }

    Ok(())
}
