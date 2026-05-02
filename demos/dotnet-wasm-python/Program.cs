using HyperlightSandbox.Api;
using HyperlightSandbox.Guest.Python;

Console.WriteLine("=== Hyperlight Wasm Python Demo (.NET 10) ===");
Console.WriteLine();

// ── Build the sandbox once and register all tools before the first run ───────
using var sandbox = new SandboxBuilder()
    .WithPythonModule()
    .Build();

// Tools must be registered before the first Run() call.
// Property names on the DTO must match the Python kwarg names (case-sensitive).
sandbox.RegisterTool<MathArgs, double>("add", args => args.a + args.b);

// ── 1. Basic Python execution ────────────────────────────────────────────────
Console.WriteLine("--- 1. Basic Python execution ---");

var result = sandbox.Run("""
    import math
    primes = [n for n in range(2, 50)
              if all(n % i != 0 for i in range(2, int(math.sqrt(n)) + 1))]
    print(f"Primes up to 50: {primes}")
    """);

Console.WriteLine(result.Stdout.TrimEnd());
Console.WriteLine($"Success: {result.Success}");
Console.WriteLine();

// ── 2. Tool registration – call a .NET function from Python ──────────────────
Console.WriteLine("--- 2. Tool registration (.NET → Python) ---");

result = sandbox.Run("""
    total = call_tool("add", a=10, b=32)
    print(f"10 + 32 = {total}")
    """);

Console.WriteLine(result.Stdout.TrimEnd());
Console.WriteLine($"Success: {result.Success}");
Console.WriteLine();

// ── 3. Snapshot / restore ────────────────────────────────────────────────────
Console.WriteLine("--- 3. Snapshot / restore ---");

// Warm up so the snapshot captures a clean Python interpreter state.
sandbox.Run("pass");
using var snapshot = sandbox.Snapshot();

sandbox.Run("x = 42");
sandbox.Restore(snapshot);

result = sandbox.Run("""
    try:
        print(x)
    except NameError as e:
        print(f"After restore: {e}")
    """);

Console.WriteLine(result.Stdout.TrimEnd());
Console.WriteLine();

Console.WriteLine("=== Demo complete ===");

// ── DTO for typed tool ───────────────────────────────────────────────────────
// Property names must match the Python kwarg names passed to call_tool().
record MathArgs(double a, double b);
