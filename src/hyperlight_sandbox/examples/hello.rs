//! Quick smoke test: load the python-sandbox AOT component and run print("hello").

use hyperlight_sandbox::{PythonSandbox, SandboxConfig};

fn main() {
    let config = SandboxConfig {
        module_path: "src/python_sandbox/python-sandbox.aot".to_string(),
        heap_size: 200 * 1024 * 1024,  // 200Mi (must be > input buffer 70Mi)
        stack_size: 100 * 1024 * 1024, // 100Mi scratch
        timeout_secs: Some(30),
    };

    println!("Creating sandbox...");
    let mut sandbox = match PythonSandbox::new(config) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to create sandbox: {e:#}");
            std::process::exit(1);
        }
    };

    println!("Running print('hello from wasm!')...");
    match sandbox.run("print('hello from wasm!')") {
        Ok(result) => {
            println!("exit_code: {}", result.exit_code);
            println!("stdout: {:?}", result.stdout);
            println!("stderr: {:?}", result.stderr);
        }
        Err(e) => {
            eprintln!("Execution failed: {e}");
            std::process::exit(1);
        }
    }
}
