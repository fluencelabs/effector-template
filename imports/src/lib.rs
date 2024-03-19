pub use ls_effector_types::*;
use marine_rs_sdk::marine;

#[marine]
#[module_import("ls_effector")]
extern "C" {
    // List files in the provided directory in the particle vault
    pub fn ls_vault(vault_path: String) -> ListResult;
}
