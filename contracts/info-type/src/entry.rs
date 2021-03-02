use core::result::Result;
use alloc::vec::Vec;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{packed::*, prelude::*},
    high_level::{load_cell_data, load_input_out_point, load_cell_type, load_input_since},
};
use crate::error::Error;

const TIMESTAMP_DATA_LEN: usize = 5;
const BLOCK_NUMBER_DATA_LEN: usize = 9;
const INDEX_STATE_CELL_DATA_LEN: usize = 2;

pub fn main() -> Result<(), Error> {
    match load_cell_type(0, Source::GroupInput)? {
        // Create the time info cell and the input info type script doesn't exist
        None => {
            load_output_type_script(|output_type_script| {
                let out_point = load_input_out_point(0, Source::Input)?;
                if output_type_script.args().as_slice() != out_point.as_slice() {
                    return Err(Error::InvalidArgument);
                }
                check_info_cell_data()
            })
        },
        // Update the time info cell and the info type scripts of input and output exist
        Some(input_type_script) => {
            load_output_type_script(|output_type_script| {
                if output_type_script.as_slice() != input_type_script.as_slice() {
                    return Err(Error::TypeOfCellsNotSame);
                }
                check_info_cells_data()
            })
        }
    }
}

// Time info cell data: index(u8) | timestamp(u32) or block number(u64)
fn check_info_cell_data() -> Result<(), Error> {
    let info_data = load_cell_data(0, Source::GroupOutput)?;
    if info_data.len() != TIMESTAMP_DATA_LEN || info_data.len() != BLOCK_NUMBER_DATA_LEN {
        return Err(Error::TimeInfoDataLenError);
    }
    Ok(())
}

fn check_info_cells_data() -> Result<(), Error> {
    let index_state_data = load_index_state_output_data()?;
    let input_info_data = load_cell_data(0, Source::GroupInput)?;
    let output_info_data = load_cell_data(0, Source::GroupOutput)?;

    if output_info_data.len() != TIMESTAMP_DATA_LEN 
        || output_info_data.len() != BLOCK_NUMBER_DATA_LEN 
        || output_info_data.len() != input_info_data.len() {
        return Err(Error::TimeInfoDataLenError);
    } 

    if output_info_data[0] != index_state_data[0] {
        return Err(Error::TimeInfoIndexNotSame);
    } 

    let since = load_input_since(0, Source::GroupInput)?;
    
    if output_info_data.len() == TIMESTAMP_DATA_LEN {
        let input_timestamp = timestamp_from_info_data(&input_info_data);
        let output_timestamp = timestamp_from_info_data(&output_info_data);

        if input_timestamp >= output_timestamp {
            return Err(Error::OutputTimeNotBigger);
        }

        let since_timestamp_base: u64 = 1 << 62;
        if since_timestamp_base + output_timestamp as u64 != since {
            return Err(Error::InvalidTimeInfoSince);
        }

    } 
    
    if output_info_data.len() == BLOCK_NUMBER_DATA_LEN {
        let input_block_number = block_number_from_info_data(&input_info_data);
        let output_block_number = block_number_from_info_data(&output_info_data);

        if input_block_number >= output_block_number {
            return Err(Error::OutputBlockNumberNotBigger);
        }

        if output_block_number != since {
            return Err(Error::InvalidTimeInfoSince);
        }
    }

    Ok(())
}

fn load_index_state_output_data() -> Result<Vec<u8>, Error> {
    let index_state_data = load_cell_data(0, Source::Output)?;
    if index_state_data.len() != INDEX_STATE_CELL_DATA_LEN {
        return Err(Error::IndexStateDataLenError);
    }
    Ok(index_state_data)
}

fn timestamp_from_info_data(info_data: &Vec<u8>) -> u32 {
    let mut timestamp_buf = [0u8; TIMESTAMP_DATA_LEN - 1];
    timestamp_buf.copy_from_slice(&info_data[1..]);
    u32::from_be_bytes(timestamp_buf)
}

fn block_number_from_info_data(info_data: &Vec<u8>) -> u64 {
    let mut block_number_buf = [0u8; BLOCK_NUMBER_DATA_LEN - 1];
    block_number_buf.copy_from_slice(&info_data[1..]);
    u64::from_be_bytes(block_number_buf)
}

fn load_output_type_script<F>(closure: F) -> Result<(), Error> 
    where F: Fn(Script) -> Result<(), Error> {
    match load_cell_type(0, Source::GroupOutput)? {
        Some(output_type_script) => closure(output_type_script),
        None => Err(Error::Encoding)
    }
}
