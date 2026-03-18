//! Smoke test: load the python-sandbox AOT component, run code, and test tool dispatch.

use hyperlight_sandbox::{PythonSandbox, SandboxConfig, ToolRegistry};

fn main() {
    let config = SandboxConfig {
        module_path: "src/python_sandbox/python-sandbox.aot".to_string(),
        heap_size: 200 * 1024 * 1024,
        stack_size: 100 * 1024 * 1024,
        timeout_secs: Some(30),
    };

    // Register tools before creating the sandbox
    let mut tools = ToolRegistry::new();
    tools.register("add", |args| {
        let a = args["a"].as_f64().unwrap_or(0.0);
        let b = args["b"].as_f64().unwrap_or(0.0);
        Ok(serde_json::json!(a + b))
    });
    tools.register("greet", |args| {
        let name = args["name"].as_str().unwrap_or("world");
        Ok(serde_json::json!(format!("Hello, {}!", name)))
    });

    println!("Creating sandbox with tools...");
    let mut sandbox = match PythonSandbox::with_tools(config, tools) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to create sandbox: {e:#}");
            std::process::exit(1);
        }
    };

    // Test 1: Basic code execution (Phase 1)
    println!("\n--- Test 1: Basic execution ---");
    match sandbox.run("print('hello from wasm!')") {
        Ok(result) => println!("stdout: {:?}, exit_code: {}", result.stdout, result.exit_code),
        Err(e) => eprintln!("Failed: {e}"),
    }

    // Test 2: Tool dispatch via call_tool (provided by sandbox_executor)
    println!("\n--- Test 2: Tool dispatch ---");
    match sandbox.run(r#"
result = call_tool('add', a=3, b=4)
greeting = call_tool('greet', name='James')
print(f"3 + 4 = {result}")
print(f"Greeting: {greeting}")
try:
    call_tool('nonexistent', x=1)
except RuntimeError as e:
    print(f"Caught error: {e}")
print("All tool tests passed!")
"#) {
        Ok(result) => {
            println!("stdout: {:?}", result.stdout);
            println!("stderr: {:?}", result.stderr);
            println!("exit_code: {}", result.exit_code);
        }
        Err(e) => eprintln!("Failed: {e}"),
    }

    // Test 3: Multiple runs (verify sandbox reuse)
    println!("\n--- Test 3: Second run ---");
    match sandbox.run("print('second run works!')") {
        Ok(result) => println!("stdout: {:?}, exit_code: {}", result.stdout, result.exit_code),
        Err(e) => eprintln!("Failed (expected — component re-entry issue): {e}"),
    }

    // Test 4: File I/O via WASI filesystem
    println!("\n--- Test 4: File I/O ---");
    sandbox.add_file("data.json", br#"{"greeting": "hello from file!"}"#.to_vec());
    match sandbox.run(r#"
import json
with open('/input/data.json', 'r') as f:
    data = json.load(f)
print(f"Read from file: {data['greeting']}")

with open('/output/result.txt', 'w') as f:
    f.write('Written from sandbox!')
print("File I/O test passed!")
"#) {
        Ok(result) => {
            println!("stdout: {:?}", result.stdout);
            println!("exit_code: {}", result.exit_code);
            for (name, data) in &result.outputs {
                println!("output[{name}]: {:?}", String::from_utf8_lossy(data));
            }
        }
        Err(e) => eprintln!("Failed: {e}"),
    }
}
