use core::result::Result;
use alloc::vec::Vec;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{packed::*, prelude::*},
    high_level::{load_cell_data, load_input_out_point, load_cell_type},
};
use crate::error::Error;

const SUM_OF_TIME_INFO_CELLS: u8 = 12;
const INDEX_STATE_CELL_DATA_LEN: usize = 2;

pub fn main() -> Result<(), Error> {
    match load_cell_type(0, Source::GroupInput)? {
        // Create the time index state cell and the input type script doesn't exist
        None => {
            load_output_type_script(|output_type_script| {
                let out_point = load_input_out_point(0, Source::Input)?;
                if output_type_script.args().as_slice() != out_point.as_slice() {
                    return Err(Error::InvalidArgument);
                }
                let _ = check_index_state_cell_data(Source::GroupOutput)?;
                Ok(())
            })
        },
        // Update the time index state cell and the type scripts of input and output exist
        Some(input_type_script) => {
            load_output_type_script(|output_type_script| {
                if output_type_script.as_slice() != input_type_script.as_slice() {
                    return Err(Error::TypeOfCellsNotSame);
                }
                check_index_state_cells_data()
            })
        }
    }
}

// Time index state cell data: index(u8) | sum_of_time_info_cells(u8)
fn check_index_state_cell_data(source: Source) -> Result<Vec<u8>, Error> {
    let data = load_cell_data(0, source)?;
    if data.len() != INDEX_STATE_CELL_DATA_LEN {
        return Err(Error::IndexStateDataLenError);
    }
    if data[0] >= SUM_OF_TIME_INFO_CELLS {
        return Err(Error::TimeIndexOutOfBound);
    }
    if data[1] != SUM_OF_TIME_INFO_CELLS {
        return Err(Error::TimeInfoAccountError);
    }
    Ok(data)
}

fn check_index_state_cells_data() -> Result<(), Error> {
    let input_data = check_index_state_cell_data(Source::GroupInput)?;
    let output_data = check_index_state_cell_data(Source::GroupOutput)?;
    if input_data[0] + 1 != output_data[0] {
        return Err(Error::TimeIndexIncreaseError);
    }
    Ok(())
}

fn load_output_type_script<F>(closure: F) -> Result<(), Error> 
    where F: Fn(Script) -> Result<(), Error> {
    match load_cell_type(0, Source::GroupOutput)? {
        Some(output_type_script) => closure(output_type_script),
        None => Err(Error::Encoding)
    }
}


