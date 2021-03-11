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
use ckb_x64_simulator::RunningSetup;
use std::collections::HashMap;

const TIME_INDEX_CELL_DATA_LEN: usize = 2;
const SUM_OF_TIME_INFO_CELLS: u8 = 12;
const MAX_CYCLES: u64 = 10_000_000;

// error numbers
const INVALID_ARGUMENT: i8 = 5;
const INDEX_STATE_TYPE_NOT_EXIST: i8 = 6;
const INDEX_STATE_DATA_LEN_ERROR: i8 = 7;
const TIME_INFO_AMOUNT_ERROR: i8 = 8;
const TIME_INDEX_OUT_OF_BOUND: i8 = 9;
const TIME_INDEX_INCREASE_ERROR: i8 = 10;

fn build_index_state_cell_data(index: u8, sum: u8) -> Bytes {
    let mut time_buf = BytesMut::with_capacity(TIME_INDEX_CELL_DATA_LEN);
    time_buf.put_u8(index);
    time_buf.put_u8(sum);
    Bytes::from(time_buf.to_vec())
}

fn build_invalid_index_state_cell_data() -> Bytes {
    let mut time_buf = BytesMut::with_capacity(3);
    for _ in 0..3 {
        time_buf.put_u8(0);
    }
    Bytes::from(time_buf.to_vec())
}

fn create_test_context(
    outputs_data: &Vec<Bytes>,
    is_type_args_error: bool,
) -> (Context, TransactionView) {
    // deploy contract
    let mut context = Context::default();
    let index_state_bin: Bytes = Loader::default().load_binary("index-state-type");
    let index_state_out_point = context.deploy_cell(index_state_bin);

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
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(lock_script.clone())
            .build(),
        Bytes::new(),
    );

    let args = if is_type_args_error {
        Bytes::new()
    } else {
        Bytes::copy_from_slice(input_out_point.as_slice().clone())
    };
    let index_state_type_script = context
        .build_script(&index_state_out_point, args)
        .expect("script");
    let index_state_type_script_dep = CellDep::new_builder()
        .out_point(index_state_out_point)
        .build();

    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();
    let outputs = vec![
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .type_(Some(index_state_type_script.clone()).pack())
            .build(),
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script)
            .build(),
    ];

    let witnesses = vec![Bytes::new()];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(index_state_type_script_dep)
        .witnesses(witnesses.pack())
        .build();
    (context, tx)
}

fn create_test_context_with_index_state_inputs(
    input_data: Bytes,
    outputs_data: &Vec<Bytes>,
    type_of_cells_not_same: bool,
) -> (Context, TransactionView) {
    // deploy contract
    let mut context = Context::default();
    let index_state_bin: Bytes = Loader::default().load_binary("index-state-type");
    let index_state_out_point = context.deploy_cell(index_state_bin);

    // deploy always_success script
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

    // prepare scripts
    let lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let normal_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .build(),
        Bytes::new(),
    );

    let args = Bytes::copy_from_slice(normal_out_point.as_slice().clone());
    let index_state_type_script = context
        .build_script(&index_state_out_point, args)
        .expect("script");
    let index_state_type_script_dep = CellDep::new_builder()
        .out_point(index_state_out_point.clone())
        .build();

    let index_state_input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .type_(Some(index_state_type_script.clone()).pack())
            .build(),
        input_data,
    );

    let inputs = vec![
        CellInput::new_builder()
            .previous_output(index_state_input_out_point.clone())
            .build(),
        CellInput::new_builder()
            .previous_output(normal_out_point)
            .build(),
    ];

    let mut outputs = if type_of_cells_not_same {
        let another_args = Bytes::copy_from_slice(index_state_input_out_point.as_slice().clone());
        let another_index_state_type_script = context
            .build_script(&index_state_out_point, another_args)
            .expect("script");
        vec![CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .type_(Some(another_index_state_type_script.clone()).pack())
            .build()]
    } else {
        vec![CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .type_(Some(index_state_type_script.clone()).pack())
            .build()]
    };
    outputs.push(
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script)
            .build(),
    );

    let witnesses = vec![Bytes::new(), Bytes::new()];

    // build transaction
    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(index_state_type_script_dep)
        .witnesses(witnesses.pack())
        .build();
    (context, tx)
}

#[test]
fn test_create_index_state_cells_success() {
    let outputs_data = vec![
        build_index_state_cell_data(0, SUM_OF_TIME_INFO_CELLS),
        Bytes::new(),
    ];
    let (mut context, tx) = create_test_context(&outputs_data, false);

    let tx = context.complete_tx(tx);
    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);

    // dump raw test tx files
    let setup = RunningSetup {
        is_lock_script: false,
        is_output: true,
        script_index: 0,
        native_binaries: HashMap::default(),
    };
    write_native_setup(
        "test_create_index_state_cells_success",
        "ckb-time-index-state-type-sim",
        &tx,
        &context,
        &setup,
    );
}

#[test]
fn test_update_index_state_cells_success() {
    let input_data = build_index_state_cell_data(1, SUM_OF_TIME_INFO_CELLS);
    let outputs_data = vec![
        build_index_state_cell_data(2, SUM_OF_TIME_INFO_CELLS),
        Bytes::new(),
    ];
    let (mut context, tx) =
        create_test_context_with_index_state_inputs(input_data, &outputs_data, false);

    let tx = context.complete_tx(tx);
    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);

    // dump raw test tx files
    let setup = RunningSetup {
        is_lock_script: false,
        is_output: true,
        script_index: 0,
        native_binaries: HashMap::default(),
    };
    write_native_setup(
        "test_update_index_state_cells_success",
        "ckb-time-index-state-type-sim",
        &tx,
        &context,
        &setup,
    );
}

#[test]
fn test_update_full_index_state_cells_success() {
    let input_data = build_index_state_cell_data(11, SUM_OF_TIME_INFO_CELLS);
    let outputs_data = vec![
        build_index_state_cell_data(0, SUM_OF_TIME_INFO_CELLS),
        Bytes::new(),
    ];
    let (mut context, tx) =
        create_test_context_with_index_state_inputs(input_data, &outputs_data, false);

    let tx = context.complete_tx(tx);
    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);

    // dump raw test tx files
    let setup = RunningSetup {
        is_lock_script: false,
        is_output: true,
        script_index: 0,
        native_binaries: HashMap::default(),
    };
    write_native_setup(
        "test_update_full_index_state_cells_success",
        "ckb-time-index-state-type-sim",
        &tx,
        &context,
        &setup,
    );
}

#[test]
fn test_error_index_state_len() {
    let outputs_data = vec![build_invalid_index_state_cell_data(), Bytes::new()];
    let (mut context, tx) = create_test_context(&outputs_data, false);
    let tx = context.complete_tx(tx);
    // run
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    let script_cell_index = 0;
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(INDEX_STATE_DATA_LEN_ERROR)
            .output_type_script(script_cell_index)
    );

    // dump raw test tx files
    let setup = RunningSetup {
        is_lock_script: false,
        is_output: true,
        script_index: 0,
        native_binaries: HashMap::default(),
    };
    write_native_setup(
        "test_error_index_state_len",
        "ckb-time-index-state-type-sim",
        &tx,
        &context,
        &setup,
    );
}

#[test]
fn test_error_info_amount() {
    let outputs_data = vec![build_index_state_cell_data(0, 10), Bytes::new()];
    let (mut context, tx) = create_test_context(&outputs_data, false);
    let tx = context.complete_tx(tx);
    // run
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    let script_cell_index = 0;
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(TIME_INFO_AMOUNT_ERROR)
            .output_type_script(script_cell_index)
    );

    // dump raw test tx files
    let setup = RunningSetup {
        is_lock_script: false,
        is_output: true,
        script_index: 0,
        native_binaries: HashMap::default(),
    };
    write_native_setup(
        "test_error_info_amount",
        "ckb-time-index-state-type-sim",
        &tx,
        &context,
        &setup,
    );
}

#[test]
fn test_error_index_out_of_bound() {
    let outputs_data = vec![
        build_index_state_cell_data(13, SUM_OF_TIME_INFO_CELLS),
        Bytes::new(),
    ];
    let (mut context, tx) = create_test_context(&outputs_data, false);
    let tx = context.complete_tx(tx);
    // run
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    let script_cell_index = 0;
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(TIME_INDEX_OUT_OF_BOUND)
            .output_type_script(script_cell_index)
    );

    // dump raw test tx files
    let setup = RunningSetup {
        is_lock_script: false,
        is_output: true,
        script_index: 0,
        native_binaries: HashMap::default(),
    };
    write_native_setup(
        "test_error_index_out_of_bound",
        "ckb-time-index-state-type-sim",
        &tx,
        &context,
        &setup,
    );
}

#[test]
fn test_error_args_invalid() {
    let outputs_data = vec![
        build_index_state_cell_data(0, SUM_OF_TIME_INFO_CELLS),
        Bytes::new(),
    ];
    let (mut context, tx) = create_test_context(&outputs_data, true);
    let tx = context.complete_tx(tx);
    // run
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    let script_cell_index = 0;
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(INVALID_ARGUMENT).output_type_script(script_cell_index)
    );

    // dump raw test tx files
    let setup = RunningSetup {
        is_lock_script: false,
        is_output: true,
        script_index: 0,
        native_binaries: HashMap::default(),
    };
    write_native_setup(
        "test_error_args_invalid",
        "ckb-time-index-state-type-sim",
        &tx,
        &context,
        &setup,
    );
}

#[test]
fn test_error_type_of_cells_not_same() {
    let input_data = build_index_state_cell_data(1, SUM_OF_TIME_INFO_CELLS);
    let outputs_data = vec![
        build_index_state_cell_data(2, SUM_OF_TIME_INFO_CELLS),
        Bytes::new(),
    ];
    let (mut context, tx) =
        create_test_context_with_index_state_inputs(input_data, &outputs_data, true);

    let tx = context.complete_tx(tx);
    // run
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    let script_cell_index = 0;
    assert_type_script_error(err, INDEX_STATE_TYPE_NOT_EXIST, script_cell_index);
}

#[test]
fn test_error_index_not_increase() {
    let input_data = build_index_state_cell_data(3, SUM_OF_TIME_INFO_CELLS);
    let outputs_data = vec![
        build_index_state_cell_data(2, SUM_OF_TIME_INFO_CELLS),
        Bytes::new(),
    ];
    let (mut context, tx) =
        create_test_context_with_index_state_inputs(input_data, &outputs_data, false);

    let tx = context.complete_tx(tx);
    // run
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    let script_cell_index = 0;
    assert_error_eq!(
        err,
        ScriptError::ValidationFailure(TIME_INDEX_INCREASE_ERROR)
            .input_type_script(script_cell_index)
    );

    // dump raw test tx files
    let setup = RunningSetup {
        is_lock_script: false,
        is_output: true,
        script_index: 0,
        native_binaries: HashMap::default(),
    };
    write_native_setup(
        "test_error_index_not_increase",
        "ckb-time-index-state-type-sim",
        &tx,
        &context,
        &setup,
    );
}
