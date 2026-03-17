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

    // Test 2: Tool dispatch (Phase 2)
    println!("\n--- Test 2: Tool dispatch ---");
    match sandbox.run(r#"
from hyperlight import call_tool
result = call_tool('add', a=3, b=4)
print(f"3 + 4 = {result}")
"#) {
        Ok(result) => println!("stdout: {:?}, exit_code: {}", result.stdout, result.exit_code),
        Err(e) => eprintln!("Failed: {e}"),
    }

    // Test 3: Multiple tool calls
    println!("\n--- Test 3: Multiple tool calls ---");
    match sandbox.run(r#"
from hyperlight import call_tool
greeting = call_tool('greet', name='James')
sum_result = call_tool('add', a=10, b=20)
print(f"{greeting} The answer is {sum_result}")
"#) {
        Ok(result) => println!("stdout: {:?}, exit_code: {}", result.stdout, result.exit_code),
        Err(e) => eprintln!("Failed: {e}"),
    }

    // Test 4: Unknown tool error handling
    println!("\n--- Test 4: Unknown tool error ---");
    match sandbox.run(r#"
from hyperlight import call_tool
try:
    call_tool('nonexistent', x=1)
except RuntimeError as e:
    print(f"Caught error: {e}")
"#) {
        Ok(result) => println!("stdout: {:?}, exit_code: {}", result.stdout, result.exit_code),
        Err(e) => eprintln!("Failed: {e}"),
    }
}
