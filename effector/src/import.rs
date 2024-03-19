use marine_rs_sdk::{marine, MountedBinaryResult};

// Here we need to import the mounted binary in the module using `host_import`
#[marine]
#[host_import]
extern "C" {
    /// Execute provided cmd as a parameters of ls.
    pub fn ls(cmd: Vec<String>) -> MountedBinaryResult;
}
