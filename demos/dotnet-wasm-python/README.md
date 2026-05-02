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

---

## Install .NET 10

### Linux (Ubuntu / Debian)

```bash
# Register the Microsoft package feed
wget https://packages.microsoft.com/config/ubuntu/$(lsb_release -rs)/packages-microsoft-prod.deb -O packages-microsoft-prod.deb
sudo dpkg -i packages-microsoft-prod.deb
rm packages-microsoft-prod.deb

# Install the SDK
sudo apt-get update && sudo apt-get install -y dotnet-sdk-10.0

# Verify
dotnet --version   # should print 10.x.x
```

### macOS (Homebrew)

```bash
brew install --cask dotnet-sdk
dotnet --version
```

### Windows

Download and run the installer from
<https://dotnet.microsoft.com/download/dotnet/10.0>, or use winget:

```powershell
winget install Microsoft.DotNet.SDK.10
dotnet --version
```

---

## Create your own sandbox from scratch

Follow these steps to build a new project that runs Python inside a
Hyperlight sandbox — no need to clone this repo.

### 1. Create a new console app

```bash
dotnet new console -n MyHyperlightDemo
cd MyHyperlightDemo
```

### 2. Add the Hyperlight NuGet packages

```bash
dotnet add package Hyperlight.HyperlightSandbox.Api        --version 0.4.0
dotnet add package Hyperlight.HyperlightSandbox.Guest.Python --version 0.4.0
```

### 3. Allow unsafe blocks (required by the native interop layer)

Open `MyHyperlightDemo.csproj` and add `<AllowUnsafeBlocks>true</AllowUnsafeBlocks>`
inside the `<PropertyGroup>`:

```xml
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net10.0</TargetFramework>
    <Nullable>enable</Nullable>
    <ImplicitUsings>enable</ImplicitUsings>
    <AllowUnsafeBlocks>true</AllowUnsafeBlocks>
  </PropertyGroup>
  <ItemGroup>
    <PackageReference Include="Hyperlight.HyperlightSandbox.Api"         Version="0.4.0" />
    <PackageReference Include="Hyperlight.HyperlightSandbox.Guest.Python" Version="0.4.0" />
  </ItemGroup>
</Project>
```

### 4. Write your first sandbox program

Replace the contents of `Program.cs` with:

```csharp
using HyperlightSandbox.Api;
using HyperlightSandbox.Guest.Python;

// Build the sandbox and register tools BEFORE the first Run() call.
using var sandbox = new SandboxBuilder()
    .WithPythonModule()
    .Build();

// (Optional) expose a .NET function to Python via call_tool()
sandbox.RegisterTool<MathArgs, double>("add", args => args.a + args.b);

// Run Python code inside the isolated sandbox
var result = sandbox.Run("""
    total = call_tool("add", a=10, b=32)
    print(f"10 + 32 = {total}")
    """);

Console.WriteLine(result.Stdout.TrimEnd());   // 10 + 32 = 42.0
Console.WriteLine($"Success: {result.Success}");

// Property names must match the Python kwarg names exactly (case-sensitive).
record MathArgs(double a, double b);
```

### 5. Run it

```bash
dotnet run
```

Expected output:

```
10 + 32 = 42.0
Success: True
```

---

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
