use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
    let library_paths = HashMap::from([(
        "mapping".to_string(),
        PathBuf::from(slint_mapping::UI_LIBRARY_DIR),
    )]);
    let config = slint_build::CompilerConfiguration::new().with_library_paths(library_paths);
    slint_build::compile_with_config("ui/dashboard.slint", config).expect("Slint build failed");
}
