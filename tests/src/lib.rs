#[macro_use]
extern crate lazy_static;

use ckb_tool::ckb_error::Error;
use ckb_tool::ckb_script::ScriptError;
use ckb_tool::ckb_types::bytes::Bytes;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use ckb_testtool::context::Context;
use ckb_standalone_debugger::transaction::{
    MockCellDep, MockInfo, MockInput, MockTransaction, ReprMockTransaction,
};
use ckb_x64_simulator::RunningSetup;
use serde_json::to_string_pretty;

use ckb_tool::ckb_types::{
    core::{DepType, TransactionView},
};

#[cfg(test)]
mod info_tests;

#[cfg(test)]
mod index_state_tests;

lazy_static! {
    static ref LOADER: Loader = Loader::default();
    static ref TX_FOLDER: PathBuf = {
        let path = LOADER.path("dumped_tests");
        if Path::new(&path).exists() {
            fs::remove_dir_all(&path).expect("remove old dir");
        }
        fs::create_dir_all(&path).expect("create test dir");
        path
    };
}

const TEST_ENV_VAR: &str = "CAPSULE_TEST_ENV";

pub enum TestEnv {
    Debug,
    Release,
}

impl FromStr for TestEnv {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(TestEnv::Debug),
            "release" => Ok(TestEnv::Release),
            _ => Err("no match"),
        }
    }
}

pub struct Loader(PathBuf);

impl Default for Loader {
    fn default() -> Self {
        let test_env = match env::var(TEST_ENV_VAR) {
            Ok(val) => val.parse().expect("test env"),
            Err(_) => TestEnv::Debug,
        };
        Self::with_test_env(test_env)
    }
}

impl Loader {
    fn with_test_env(env: TestEnv) -> Self {
        let load_prefix = match env {
            TestEnv::Debug => "debug",
            TestEnv::Release => "release",
        };
        let dir = env::current_dir().unwrap();
        let mut base_path = PathBuf::new();
        base_path.push(dir);
        base_path.push("..");
        base_path.push("build");
        base_path.push(load_prefix);
        Loader(base_path)
    }

    pub fn path(&self, name: &str) -> PathBuf {
        let mut path = self.0.clone();
        path.push(name);
        path
    }

    pub fn load_binary(&self, name: &str) -> Bytes {
        let mut path = self.0.clone();
        path.push(name);
        fs::read(path).expect("binary").into()
    }
}

pub fn assert_type_script_error(err: Error, error_code: i8, script_cell_index: usize) {
    let input_type_error = Into::<Error>::into(
        ScriptError::ValidationFailure(error_code).input_type_script(script_cell_index),
    )
    .to_string();
    let output_type_error = Into::<Error>::into(
        ScriptError::ValidationFailure(error_code).output_type_script(script_cell_index),
    )
    .to_string();
    let error = Into::<Error>::into(err).to_string();
    let result = input_type_error == error || output_type_error == error;
    assert!(result);
}

fn create_test_folder(name: &str) -> PathBuf {
    let mut path = TX_FOLDER.clone();
    path.push(&name);
    fs::create_dir_all(&path).expect("create folder");
    path
}

fn build_mock_transaction(tx: &TransactionView, context: &Context) -> MockTransaction {
    let mock_inputs = tx
        .inputs()
        .into_iter()
        .map(|input| {
            let (output, data) = context
                .get_cell(&input.previous_output())
                .expect("get cell");
            MockInput {
                input,
                output,
                data,
                header: None,
            }
        })
        .collect();
    let mock_cell_deps = tx
        .cell_deps()
        .into_iter()
        .map(|cell_dep| {
            if cell_dep.dep_type() == DepType::DepGroup.into() {
                panic!("Implement dep group support later!");
            }
            let (output, data) = context.get_cell(&cell_dep.out_point()).expect("get cell");
            MockCellDep {
                cell_dep,
                output,
                data,
                header: None,
            }
        })
        .collect();
    let mock_info = MockInfo {
        inputs:      mock_inputs,
        cell_deps:   mock_cell_deps,
        header_deps: vec![],
    };
    MockTransaction {
        mock_info,
        tx: tx.data(),
    }
}

pub fn write_native_setup(
    test_name: &str,
    binary_name: &str,
    tx: &TransactionView,
    context: &Context,
    setup: &RunningSetup,
) {
    let folder = create_test_folder(test_name);
    let mock_tx = build_mock_transaction(&tx, &context);
    let repr_tx: ReprMockTransaction = mock_tx.into();
    let tx_json = to_string_pretty(&repr_tx).expect("serialize to json");
    fs::write(folder.join("tx.json"), tx_json).expect("write tx to local file");
    let setup_json = to_string_pretty(setup).expect("serialize to json");
    fs::write(folder.join("setup.json"), setup_json).expect("write setup to local file");
    fs::write(
        folder.join("cmd"),
        format!(
            "CKB_TX_FILE=\"{}\" CKB_RUNNING_SETUP=\"{}\" \"{}\"",
            folder.join("tx.json").to_str().expect("utf8"),
            folder.join("setup.json").to_str().expect("utf8"),
            Loader::default().path(binary_name).to_str().expect("utf8")
        ),
    )
    .expect("write cmd to local file");
}
