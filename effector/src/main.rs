#![feature(try_blocks)]
#![feature(assert_matches)]
#![allow(improper_ctypes)]
#![allow(non_snake_case)]

mod import;
mod utils;

use eyre::{eyre, Result};
use ls_effector_types::*;
use marine_rs_sdk::marine;
use marine_rs_sdk::module_manifest;
use marine_rs_sdk::WasmLoggerBuilder;

use crate::import::ls;
use crate::utils::inject_vault;

module_manifest!();

pub fn main() {
    WasmLoggerBuilder::new()
        .with_log_level(log::LevelFilter::Debug)
        .build()
        .unwrap();
}

fn run_ls(path: String) -> Result<String> {
    let cmd = vec!["-1".into(), path.clone()];
    log::debug!("Running ls with arguments: {:?}", cmd);
    let result = ls(cmd.clone());
    log::debug!("Got result: {:?}", result.stringify());

    result
        .into_std()
        .ok_or(eyre!("stdout or stderr contains non valid UTF8 string"))?
        .map_err(|e| eyre!("ls call failed \n{:?}: {}", cmd.join(" "), e))
}

// Provide list of files in the provided directory
#[marine]
pub fn list_vault(vault_path: String) -> ListResult {
    list_vault_impl(vault_path).into()
}

fn list_vault_impl(vault_path: String) -> Result<Vec<String>> {
    let vault_path = inject_vault(&vault_path)?;
    let result = run_ls(vault_path)?;
    Ok(result.lines().map(|s| s.to_string()).collect())
}

#[test_env_helpers::before_each]
#[test_env_helpers::after_each]
#[test_env_helpers::after_all]
#[cfg(test)]
mod tests {
    use marine_rs_sdk_test::{marine_test, CallParameters};
    use std::fs::File;

    // Here we provide constant values for particle parameters.
    // They are required, since they're used to construct the correct path to the particle vault.
    const PARTICLE_ID: &str = "test_id";
    const TOKEN: &str = "token";

    // This is the path to the vault directory. Note that's a directory not for a single particle,
    // but for all particles.
    const VAULT_TEMP: &str = "./test_artifacts/temp";
    // On the other hand, this is a vault of the specific particle. Note that it's a subdirectory
    // of `VAULT_TEMP` and contains `PARTICLE_ID` and `TOKEN` in its file name.
    const PARTICLE_VAULT: &str = "./test_artifacts/temp/test_id-token";

    // Here, since we work this the filesystem in tests, we need to prepare directory
    // structure manually for testing. On deployment, all the directories will be created automatically.
    //
    // We need to clear manually after each run because it's impossible to set a temporary directory
    // as a module directory due to wasm limitations.
    fn before_each() {
        std::fs::create_dir_all(PARTICLE_VAULT).expect(&format!("create {PARTICLE_VAULT} failed"));
    }

    fn after_each() {
        std::fs::remove_dir_all(PARTICLE_VAULT).expect(&format!("remove {PARTICLE_VAULT} failed"));
    }
    fn after_all() {
        std::fs::remove_dir_all(VAULT_TEMP).expect(&format!("remove {VAULT_TEMP} failed"));
    }

    fn particle_cp() -> CallParameters {
        let mut cp = CallParameters::default();
        cp.particle.id = PARTICLE_ID.to_string();
        cp.particle.token = TOKEN.to_string();
        cp
    }

    // Test plan:
    // - Create files in the particle vault
    // - Call `list_vault` function of the tested module
    // - Compare that the result contains the expected files
    #[marine_test(config_path = "../test_artifacts/Config.toml")]
    fn test_ls(ls: marine_test_env::ls_effector::ModuleInterface) {
        // To enable logging in tests, you can use the following code.
        // Note that you also may require to set the environment variable WASM_LOG=debug.
        let _ = ::env_logger::builder()
            .filter_level(log::LevelFilter::Off)
            .filter_module("ls_effector", log::LevelFilter::Debug)
            // This module contains advanced information on the module execution, set to Debug
            // if you want to see the detailed logs.
            .filter_module("wasmer_interface_types_fl", log::LevelFilter::Off)
            .filter_module("marine_core", log::LevelFilter::Off)
            .is_test(true)
            .try_init();

        let file_names = vec!["test_file", "test_file2"];
        for name in &file_names {
            let file = format!("{}/{}", PARTICLE_VAULT, name);
            File::create(&file).unwrap();
        }

        let cp = particle_cp();

        let result = ls.list_vault_cp("/tmp/vault/test_id-token".to_string(), cp);
        assert!(result.success, "got {:?}", result);
        assert!(
            file_names
                .iter()
                .all(|e| result.result.contains(&e.to_string())),
            "expected list: {file_names:?}, got list: {:?}",
            result.result
        );
    }
}
