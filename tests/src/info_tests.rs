use super::*;
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_tool::ckb_error::assert_error_eq;
use ckb_tool::ckb_script::ScriptError;
use ckb_tool::ckb_types::{
    bytes::{BufMut, Bytes, BytesMut},
    core::{TransactionBuilder, TransactionView},
    packed::*,
    prelude::*,
};

const TIME_INDEX_CELL_DATA_LEN: usize = 2;
const SUM_OF_TIME_INFO_CELLS: u8 = 12;
const TIMESTAMP_DATA_LEN: usize = 5;
const BLOCK_NUMBER_DATA_LEN: usize = 9;

const MAX_CYCLES: u64 = 10_000_000;

// error numbers
const INVALID_ARGUMENT: i8 = 5;
const TIME_INFO_DATA_LEN_ERROR: i8 = 6;
const INDEX_STATE_DATA_LEN_ERROR: i8 = 7;
const TIME_INFO_TYPE_NOT_EXIST: i8 = 8;
const TIME_INFO_INDEX_NOT_SAME: i8 = 9;
const OUTPUT_TIMESTAMP_NOT_BIGGER: i8 = 10;
const OUTPUT_BLOCK_NUMBER_NOT_BIGGER: i8 = 11;
const INVALID_TIME_INFO_SINCE: i8 = 12;

fn build_index_state_cell_data(index: u8, is_data_len_err: bool) -> Bytes {
    let mut time_buf = BytesMut::with_capacity(TIME_INDEX_CELL_DATA_LEN);
    time_buf.put_u8(index);
    if !is_data_len_err {
        time_buf.put_u8(SUM_OF_TIME_INFO_CELLS);
    }
    Bytes::from(time_buf.to_vec())
}

struct TimeData {
    timestamp: u32,
    block_number: u64,
}
fn build_time_info_cell_data(index: u8, time: TimeData) -> Bytes {
    if time.timestamp > 0 {
        let mut time_buf = BytesMut::with_capacity(TIMESTAMP_DATA_LEN);
        time_buf.put_u8(index);
        time_buf.put_u32(time.timestamp);
        Bytes::from(time_buf.to_vec())
    } else if time.block_number > 0 {
        let mut time_buf = BytesMut::with_capacity(BLOCK_NUMBER_DATA_LEN);
        time_buf.put_u8(index);
        time_buf.put_u64(time.block_number);
        Bytes::from(time_buf.to_vec())
    } else {
        Bytes::new()
    }
}

fn create_test_context(
    outputs_data: &Vec<Bytes>,
    is_type_args_error: bool,
) -> (Context, TransactionView) {
    // deploy contract
    let mut context = Context::default();
    let index_state_bin: Bytes = Loader::default().load_binary("index-state-type");
    let index_state_out_point = context.deploy_cell(index_state_bin);

    let info_bin: Bytes = Loader::default().load_binary("info-type");
    let info_out_point = context.deploy_cell(info_bin);

    // deploy always_success script
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

    // prepare scripts
    let lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    // prepare cells
    let normal_input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .build(),
        Bytes::new(),
    );

    let index_state_type_script = context
        .build_script(
            &index_state_out_point,
            Bytes::copy_from_slice(normal_input_out_point.as_slice().clone()),
        )
        .expect("script");
    let index_state_type_script_dep = CellDep::new_builder()
        .out_point(index_state_out_point)
        .build();

    let args = if is_type_args_error {
        Bytes::new()
    } else {
        Bytes::copy_from_slice(normal_input_out_point.as_slice().clone())
    };
    let info_type_script = context.build_script(&info_out_point, args).expect("script");
    let info_type_script_dep = CellDep::new_builder().out_point(info_out_point).build();

    let index_state_input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .type_(Some(index_state_type_script.clone()).pack())
            .build(),
        build_index_state_cell_data(1, false),
    );

    let inputs = vec![
        CellInput::new_builder()
            .previous_output(normal_input_out_point)
            .build(),
        CellInput::new_builder()
            .previous_output(index_state_input_out_point)
            .build(),
    ];

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .type_(Some(index_state_type_script.clone()).pack())
            .build(),
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script)
            .type_(Some(info_type_script.clone()).pack())
            .build(),
    ];

    let witnesses = vec![Bytes::new(), Bytes::new()];

    // build transaction
    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(index_state_type_script_dep)
        .cell_dep(info_type_script_dep)
        .witnesses(witnesses.pack())
        .build();
    (context, tx)
}

fn create_test_context_with_info_inputs(
    inputs_data: &Vec<Bytes>,
    outputs_data: &Vec<Bytes>,
    since: u64,
    type_of_cells_not_same: bool,
) -> (Context, TransactionView) {
    // deploy contract
    let mut context = Context::default();
    let index_state_bin: Bytes = Loader::default().load_binary("index-state-type");
    let index_state_out_point = context.deploy_cell(index_state_bin);

    let info_bin: Bytes = Loader::default().load_binary("info-type");
    let info_out_point = context.deploy_cell(info_bin);

    // deploy always_success script
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

    // prepare scripts
    let lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let normal_input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .build(),
        Bytes::new(),
    );

    let args = Bytes::copy_from_slice(normal_input_out_point.as_slice().clone());
    let index_state_type_script = context
        .build_script(&index_state_out_point, args.clone())
        .expect("script");
    let index_state_type_script_dep = CellDep::new_builder()
        .out_point(index_state_out_point.clone())
        .build();

    let info_type_script = context.build_script(&info_out_point, args).expect("script");
    let info_type_script_dep = CellDep::new_builder()
        .out_point(info_out_point.clone())
        .build();

    let index_state_input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .type_(Some(index_state_type_script.clone()).pack())
            .build(),
        inputs_data[0].clone(),
    );

    let info_input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .type_(Some(info_type_script.clone()).pack())
            .build(),
        inputs_data[1].clone(),
    );

    let inputs = vec![
        CellInput::new_builder()
            .previous_output(index_state_input_out_point.clone())
            .build(),
        CellInput::new_builder()
            .previous_output(info_input_out_point.clone())
            .since(since.pack())
            .build(),
    ];

    let mut outputs = vec![CellOutput::new_builder()
        .capacity(500u64.pack())
        .lock(lock_script.clone())
        .type_(Some(index_state_type_script.clone()).pack())
        .build()];

    if type_of_cells_not_same {
        let another_args = Bytes::copy_from_slice(info_input_out_point.as_slice().clone());
        let another_info_type_script = context
            .build_script(&info_out_point, another_args)
            .expect("script");
        outputs.push(
            CellOutput::new_builder()
                .capacity(500u64.pack())
                .lock(lock_script.clone())
                .type_(Some(another_info_type_script.clone()).pack())
                .build(),
        );
    } else {
        outputs.push(
            CellOutput::new_builder()
                .capacity(500u64.pack())
                .lock(lock_script.clone())
                .type_(Some(info_type_script.clone()).pack())
                .build(),
        );
    };

    let witnesses = vec![Bytes::new(), Bytes::new()];

    // build transaction
    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(index_state_type_script_dep)
        .cell_dep(info_type_script_dep)
        .witnesses(witnesses.pack())
        .build();
    (context, tx)
}

#[test]
fn test_create_info_timestamp_cells_success() {
    let outputs_data = vec![
        build_index_state_cell_data(2, false),
        build_time_info_cell_data(
            2,
            TimeData {
                timestamp: 1614828683,
                block_number: 0,
            },
        ),
    ];
    let (mut context, tx) = create_test_context(&outputs_data, false);

    let tx = context.complete_tx(tx);
    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_create_info_block_number_cells_success() {
    let outputs_data = vec![
        build_index_state_cell_data(2, false),
        build_time_info_cell_data(
            2,
            TimeData {
                timestamp: 0,
                block_number: 10000,
            },
        ),
    ];
    let (mut context, tx) = create_test_context(&outputs_data, false);

    let tx = context.complete_tx(tx);
    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_update_info_timestamp_cells_success() {
    let inputs_data = vec![
        build_index_state_cell_data(6, false),
        build_time_info_cell_data(
            6,
            TimeData {
                timestamp: 1614828683,
                block_number: 0,
            },
        ),
    ];
    let outputs_data = vec![
        build_index_state_cell_data(7, false),
        build_time_info_cell_data(
            7,
            TimeData {
                timestamp: 1614829080,
                block_number: 0,
            },
        ),
    ];
    let since_timestamp_base: u64 = 1 << 62;
    let timestamp: u64 = 1614829080;
    let since = since_timestamp_base + timestamp;
    let (mut context, tx) =
        create_test_context_with_info_inputs(&inputs_data, &outputs_data, since, false);

    let tx = context.complete_tx(tx);
    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_update_info_block_number_cells_success() {
    let inputs_data = vec![
        build_index_state_cell_data(11, false),
        build_time_info_cell_data(
            11,
            TimeData {
                timestamp: 0,
                block_number: 10000,
            },
        ),
    ];
    let outputs_data = vec![
        build_index_state_cell_data(0, false),
        build_time_info_cell_data(
            0,
            TimeData {
                timestamp: 0,
                block_number: 10003,
            },
        ),
    ];
    let since: u64 = 10003;
    let (mut context, tx) =
        create_test_context_with_info_inputs(&inputs_data, &outputs_data, since, false);

    let tx = context.complete_tx(tx);
    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_create_info_cells_invalid_args_error() {
    let outputs_data = vec![
        build_index_state_cell_data(2, false),
        build_time_info_cell_data(
            2,
            TimeData {
                timestamp: 0,
                block_number: 10000,
            },
        ),
    ];
    let (mut context, tx) = create_test_context(&outputs_data, true);

    let tx = context.complete_tx(tx);

    // run
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    let script_cell_index = 1;
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(INVALID_ARGUMENT).output_type_script(script_cell_index)
    );
}

#[test]
fn test_create_info_cell_data_len_error() {
    let outputs_data = vec![
        build_index_state_cell_data(2, false),
        build_time_info_cell_data(
            2,
            TimeData {
                timestamp: 0,
                block_number: 0,
            },
        ),
    ];
    let (mut context, tx) = create_test_context(&outputs_data, false);

    let tx = context.complete_tx(tx);

    // run
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    let script_cell_index = 1;
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(TIME_INFO_DATA_LEN_ERROR)
            .output_type_script(script_cell_index)
    );
}

#[test]
fn test_index_state_cell_data_len_error() {
    let outputs_data = vec![
        build_index_state_cell_data(2, true),
        build_time_info_cell_data(
            2,
            TimeData {
                timestamp: 0,
                block_number: 1000,
            },
        ),
    ];
    let (mut context, tx) = create_test_context(&outputs_data, false);

    let tx = context.complete_tx(tx);

    // run
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    let script_cell_index = 1;
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(INDEX_STATE_DATA_LEN_ERROR)
            .input_type_script(script_cell_index)
    );
}

#[test]
fn test_info_type_not_exist_error() {
    let inputs_data = vec![
        build_index_state_cell_data(11, false),
        build_time_info_cell_data(
            11,
            TimeData {
                timestamp: 0,
                block_number: 10000,
            },
        ),
    ];
    let outputs_data = vec![
        build_index_state_cell_data(0, false),
        build_time_info_cell_data(
            0,
            TimeData {
                timestamp: 0,
                block_number: 10003,
            },
        ),
    ];
    let since: u64 = 10003;
    let (mut context, tx) =
        create_test_context_with_info_inputs(&inputs_data, &outputs_data, since, true);

    let tx = context.complete_tx(tx);

    // run
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    let script_cell_index = 1;
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(TIME_INFO_TYPE_NOT_EXIST)
            .output_type_script(script_cell_index)
    );
}

#[test]
fn test_info_index_not_same_error() {
    let inputs_data = vec![
        build_index_state_cell_data(11, false),
        build_time_info_cell_data(
            11,
            TimeData {
                timestamp: 0,
                block_number: 10000,
            },
        ),
    ];
    let outputs_data = vec![
        build_index_state_cell_data(0, false),
        build_time_info_cell_data(
            1,
            TimeData {
                timestamp: 0,
                block_number: 10003,
            },
        ),
    ];
    let since: u64 = 10003;
    let (mut context, tx) =
        create_test_context_with_info_inputs(&inputs_data, &outputs_data, since, false);

    let tx = context.complete_tx(tx);

    // run
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    let script_cell_index = 1;
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(TIME_INFO_INDEX_NOT_SAME)
            .input_type_script(script_cell_index)
    );
}

#[test]
fn test_output_block_number_not_bigger_error() {
    let inputs_data = vec![
        build_index_state_cell_data(11, false),
        build_time_info_cell_data(
            11,
            TimeData {
                timestamp: 0,
                block_number: 10000,
            },
        ),
    ];
    let outputs_data = vec![
        build_index_state_cell_data(0, false),
        build_time_info_cell_data(
            0,
            TimeData {
                timestamp: 0,
                block_number: 999,
            },
        ),
    ];
    let since: u64 = 999;
    let (mut context, tx) =
        create_test_context_with_info_inputs(&inputs_data, &outputs_data, since, false);

    let tx = context.complete_tx(tx);

    // run
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    let script_cell_index = 1;
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(OUTPUT_BLOCK_NUMBER_NOT_BIGGER)
            .input_type_script(script_cell_index)
    );
}

#[test]
fn test_output_block_number_since_error() {
    let inputs_data = vec![
        build_index_state_cell_data(11, false),
        build_time_info_cell_data(
            11,
            TimeData {
                timestamp: 0,
                block_number: 10000,
            },
        ),
    ];
    let outputs_data = vec![
        build_index_state_cell_data(0, false),
        build_time_info_cell_data(
            0,
            TimeData {
                timestamp: 0,
                block_number: 10004,
            },
        ),
    ];
    let since: u64 = 10030;
    let (mut context, tx) =
        create_test_context_with_info_inputs(&inputs_data, &outputs_data, since, false);

    let tx = context.complete_tx(tx);

    // run
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    let script_cell_index = 1;
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(INVALID_TIME_INFO_SINCE)
            .input_type_script(script_cell_index)
    );
}

#[test]
fn test_output_timestamp_not_bigger_error() {
    let inputs_data = vec![
        build_index_state_cell_data(11, false),
        build_time_info_cell_data(
            11,
            TimeData {
                timestamp: 1614829080,
                block_number: 0,
            },
        ),
    ];
    let outputs_data = vec![
        build_index_state_cell_data(0, false),
        build_time_info_cell_data(
            0,
            TimeData {
                timestamp: 1614829080,
                block_number: 0,
            },
        ),
    ];
    let since: u64 = 1614829080;
    let (mut context, tx) =
        create_test_context_with_info_inputs(&inputs_data, &outputs_data, since, false);

    let tx = context.complete_tx(tx);

    // run
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    let script_cell_index = 1;
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(OUTPUT_TIMESTAMP_NOT_BIGGER)
            .input_type_script(script_cell_index)
    );
}

#[test]
fn test_output_timestamp_since_error() {
    let inputs_data = vec![
        build_index_state_cell_data(11, false),
        build_time_info_cell_data(
            11,
            TimeData {
                timestamp: 1614829080,
                block_number: 0,
            },
        ),
    ];
    let outputs_data = vec![
        build_index_state_cell_data(0, false),
        build_time_info_cell_data(
            0,
            TimeData {
                timestamp: 1614829880,
                block_number: 0,
            },
        ),
    ];
    let since: u64 = 1614829580;
    let (mut context, tx) =
        create_test_context_with_info_inputs(&inputs_data, &outputs_data, since, false);

    let tx = context.complete_tx(tx);

    // run
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    let script_cell_index = 1;
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(INVALID_TIME_INFO_SINCE)
            .input_type_script(script_cell_index)
    );
}
