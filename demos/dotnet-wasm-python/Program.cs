using HyperlightSandbox.Api;
using HyperlightSandbox.Guest.Python;

Console.WriteLine("=== Hyperlight Wasm Python Demo (.NET 10) ===");
Console.WriteLine();

// ── 1. Basic Python execution ────────────────────────────────────────────────
Console.WriteLine("--- 1. Basic Python execution ---");

using var sandbox = new SandboxBuilder()
    .WithPythonModule()
    .Build();

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

sandbox.RegisterTool<MathArgs, double>("add", args => args.A + args.B);

result = sandbox.Run("""
    total = call_tool("add", a=10, b=32)
    print(f"10 + 32 = {total}")
    """);

Console.WriteLine(result.Stdout.TrimEnd());
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
record MathArgs(double A, double B);
