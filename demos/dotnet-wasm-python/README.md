# Hyperlight Wasm Python Demo (.NET 10)

A small .NET 10 console application that demonstrates the
[Hyperlight NuGet packages](https://www.nuget.org/packages?q=Hyperlight)
running Python code inside a **hardware-isolated Wasm sandbox**.

It is based on the [hyperlight-sandbox quick start](https://github.com/hyperlight-dev/hyperlight-sandbox#quick-start).

## NuGet packages used

| Package | Version | Purpose |
|---|---|---|
| [`Hyperlight.HyperlightSandbox.Api`](https://www.nuget.org/packages/Hyperlight.HyperlightSandbox.Api) | 0.4.0 | High-level sandbox API |
| [`Hyperlight.HyperlightSandbox.Guest.Python`](https://www.nuget.org/packages/Hyperlight.HyperlightSandbox.Guest.Python) | 0.4.0 | Pre-built Python AOT guest module |

## Prerequisites

* [.NET 10 SDK](https://dotnet.microsoft.com/download/dotnet/10.0)
* Linux with [KVM](https://help.ubuntu.com/community/KVM/Installation) **or** Windows with [Hyper-V / MSHV](https://learn.microsoft.com/en-us/virtualization/hyper-v_on_windows/)

## What the demo shows

1. **Basic Python execution** – runs a Python snippet inside a micro-VM and prints
   the primes up to 50.
2. **Tool registration** – registers a .NET `add` function that Python code calls
   via `call_tool("add", a=10, b=32)`.
3. **Snapshot / restore** – takes a checkpoint of the sandbox state and shows that
   variables created after the snapshot disappear on restore (~2 ms warm start vs
   ~2.5 s cold start).

## Run

```bash
cd demos/dotnet-wasm-python
dotnet run
```

### Expected output

```
=== Hyperlight Wasm Python Demo (.NET 10) ===

--- 1. Basic Python execution ---
Primes up to 50: [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47]
Success: True

--- 2. Tool registration (.NET → Python) ---
10 + 32 = 42.0

--- 3. Snapshot / restore ---
After restore: name 'x' is not defined

=== Demo complete ===
```
