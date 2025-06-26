# iggy‑embedded – **Build Guide (Windows)**

This short guide shows how to set up the Rust **esp** toolchain, compile the
firmware, and—in case you *do* have a board—flash and monitor it.  If you do
**not** own an ESP32 yet, you can still make sure the code compiles;

---

## 1 - Install Rust

Open **PowerShell** *outside* the project folder and run:

```powershell
Invoke-WebRequest -Uri https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe `
                 -OutFile rustup-init.exe
./rustup-init.exe -y   
```

Close & reopen PowerShell, then check:

```powershell
rustc --version    # should print something like "rustc 1.79.0"
```

---

## 2 - Install the **esp** toolchain

```powershell
cargo +stable install espup --locked --force   # installs espup CLI
espup install --targets esp32                  # adds GCC Xtensa & rustc-fork
```

This creates a Rust toolchain named **`esp`**.

---

## 3 - Build the firmware (no hardware required)

```powershell
cd path\to\iggy-embedded

# Fast type-check only
cargo check

# Full firmware (release profile)
cargo build --release
```

The binary is placed in:

```
target\xtensa-esp32-none-elf\release\esp32.elf
```

### Why **not** `cargo run`?

* When the *runner* is **enabled** it tries to flash the board via
  `espflash` → fails if no serial port.
* When the *runner* is **disabled**, Cargo will attempt to *execute*
  `esp32.elf` on Windows → `"%1 is not a valid Win32 application"`.

So, without hardware use **`cargo check`** or **`cargo build`** only.

---
