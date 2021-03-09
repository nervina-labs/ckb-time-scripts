use ckb_tool::ckb_error::Error;
use ckb_tool::ckb_script::ScriptError;
use ckb_tool::ckb_types::bytes::Bytes;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

#[cfg(test)]
mod info_tests;

#[cfg(test)]
mod index_state_tests;

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
