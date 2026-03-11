# Coupon Generator App — Developer Instructions

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Tech Stack & Why](#2-tech-stack--why)
3. [Project Structure](#3-project-structure)
4. [Development Methodology](#4-development-methodology)
5. [Environment Setup](#5-environment-setup)
6. [Phase 1 — Project Scaffolding](#6-phase-1--project-scaffolding)
7. [Phase 2 — Coupon Generator Module (TDD)](#7-phase-2--coupon-generator-module-tdd)
8. [Phase 3 — Excel Exporter Module (TDD)](#8-phase-3--excel-exporter-module-tdd)
9. [Phase 4 — GUI Module](#9-phase-4--gui-module)
10. [Phase 5 — Integration & Manual Testing](#10-phase-5--integration--manual-testing)
11. [Phase 6 — Build for Distribution](#11-phase-6--build-for-distribution)
12. [Appendix A — Glossary](#appendix-a--glossary)
13. [Appendix B — Troubleshooting](#appendix-b--troubleshooting)

---

## 1. Project Overview

### What are we building?

A desktop application that generates unique, random coupon codes and exports them to Excel files. The app runs as a standalone `.exe` (Windows) or `.app` (macOS) with no installation required.

### Feature Requirements

| # | Feature | Details |
|---|---------|---------|
| F1 | Coupon generation | Generate random coupon codes based on a user-provided prefix |
| F2 | Fixed length | Every coupon code is exactly **10 characters** long |
| F3 | Character set | Lowercase letters (`a–z`) and digits (`0–9`) only. Input is case-insensitive (e.g., prefix `SAN` becomes `san`) |
| F4 | Uniqueness | Every generated coupon in a batch must be unique — zero duplicates |
| F5 | Parallel generation | Use Rust's parallelism for fast generation, even for large batches (50k+) |
| F6 | Excel export | Export coupons to `.xlsx` files |
| F7 | File splitting | If more than 10,000 coupons, automatically split across multiple Excel files (10,000 per file) |
| F8 | Simple GUI | A native window with two inputs (prefix, count) and a generate button |
| F9 | Standalone binary | Compiles to a single executable — no runtime or installer needed |

### Example Output

If the user enters prefix `SAN` and count `5`, the app might produce:

```
san7k2m8x1
sanp3qt9f0
san5nwb2y8
sanj6ra1c4
sand0h7e3x
```

Each code is 10 characters: the 3-character prefix `san` + 7 random alphanumeric characters.

---

## 2. Tech Stack & Why

| Crate | Purpose | Why this one? |
|-------|---------|---------------|
| **rayon** `1.10` | Parallel iteration | Industry standard for data parallelism in Rust. Turns a regular iterator into a parallel one with a single method call. |
| **dashmap** `6` | Thread-safe HashSet (`DashSet`) | Lets multiple threads insert coupons concurrently without manual lock management. |
| **rand** `0.8` | Random number generation | The standard randomness crate in Rust. Each thread gets its own RNG — no contention. |
| **rust_xlsxwriter** `0.79` | Excel file creation | Pure Rust, no external dependencies. Creates native `.xlsx` files. |
| **eframe** `0.29` / **egui** | Native GUI | Immediate-mode GUI. Compiles to a single binary on Windows and macOS. No web runtime, no Electron bloat. |
| **dirs** `5` | OS directory paths | Finds the user's Desktop folder cross-platform. |
| **tempfile** `3` *(dev only)* | Temporary directories for tests | Creates temp folders that auto-delete after tests run. |

---

## 3. Project Structure

```
coupon-generator/
├── Cargo.toml                  # Dependencies and build config
├── src/
│   ├── main.rs                 # Entry point — launches the GUI
│   ├── generator.rs            # Core logic: coupon generation
│   ├── exporter.rs             # Excel export logic
│   └── ui.rs                   # GUI layout and interaction
└── tests/
    ├── generator_tests.rs      # Unit tests for generator
    └── exporter_tests.rs       # Unit tests for exporter
```

### Module Responsibilities

Each module has one job. They don't know about each other (except `ui.rs`, which calls both).

- **`generator.rs`** — Takes a prefix and count, returns a `Vec<String>` of unique coupons. Knows nothing about files or UI.
- **`exporter.rs`** — Takes a slice of strings and a path, writes Excel files. Knows nothing about how coupons were made.
- **`ui.rs`** — Draws the window, reads user input, calls generator then exporter, shows status messages.
- **`main.rs`** — Three lines: configure the window and launch it.

---

## 4. Development Methodology

We use **Test-Driven Development (TDD)** with the **Red → Green → Refactor** cycle.

### What does that mean in practice?

```
┌──────────────────────────────────────────────────────┐
│                                                      │
│   RED: Write a test for behavior that doesn't        │
│         exist yet. Run it. It MUST fail.             │
│                           │                          │
│                           ▼                          │
│   GREEN: Write the minimum code to make the          │
│           test pass. Nothing more.                   │
│                           │                          │
│                           ▼                          │
│   REFACTOR: Clean up the code while keeping          │
│              all tests green.                        │
│                           │                          │
│                           ▼                          │
│            Loop back to RED for the next behavior    │
│                                                      │
└──────────────────────────────────────────────────────┘
```

### Rules to follow

1. **Never write production code without a failing test first.**
2. **Run `cargo test` after every change.** If a test breaks, fix it before moving on.
3. **Commit after every Green and every Refactor step.** Small commits with clear messages.
4. **Tests are documentation.** A future developer should understand what a function does just by reading its tests.

---

## 5. Environment Setup

### Prerequisites

1. **Install Rust** via [rustup.rs](https://rustup.rs/):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
   On Windows, download and run `rustup-init.exe` from the same site.

2. **Verify installation:**
   ```bash
   rustc --version    # Should print 1.7x or higher
   cargo --version    # Should print same version
   ```

3. **IDE recommendation:** VS Code with the `rust-analyzer` extension. It gives you inline errors, autocompletion, and one-click test running.

4. **(macOS only)** For building `.app` bundles later:
   ```bash
   cargo install cargo-bundle
   ```

---

## 6. Phase 1 — Project Scaffolding

**Goal:** An empty project that compiles and has the right structure.

### Step 1.1 — Create the project

```bash
cargo init coupon-generator
cd coupon-generator
```

### Step 1.2 — Replace `Cargo.toml` with this

```toml
[package]
name = "coupon-generator"
version = "0.1.0"
edition = "2021"

[dependencies]
rayon = "1.10"
dashmap = "6"
rand = "0.8"
rust_xlsxwriter = "0.79"
eframe = "0.29"
dirs = "5"

[dev-dependencies]
tempfile = "3"

[profile.release]
opt-level = 3
lto = true
```

> **What does `[profile.release]` do?** It tells the Rust compiler to maximize optimization (`opt-level = 3`) and enable link-time optimization (`lto = true`) when building the final binary. This makes the app significantly faster and the executable smaller.

### Step 1.3 — Create the module files

Create these empty files:

```bash
touch src/generator.rs
touch src/exporter.rs
touch src/ui.rs
mkdir tests
touch tests/generator_tests.rs
touch tests/exporter_tests.rs
```

### Step 1.4 — Set up `main.rs`

Replace the contents of `src/main.rs` with:

```rust
pub mod generator;
pub mod exporter;
mod ui;

fn main() {
    println!("Coupon Generator — modules loaded.");
}
```

> **Why `pub mod` for generator and exporter but just `mod` for ui?** The `pub` keyword makes those modules accessible from the `tests/` directory. The `ui` module is only used internally by `main.rs`, so it stays private.

### Step 1.5 — Verify it compiles

```bash
cargo build
```

If you see `Finished` with no errors, you're good. Commit this.

```bash
git init
git add .
git commit -m "Phase 1: project scaffolding with all dependencies"
```

---

## 7. Phase 2 — Coupon Generator Module (TDD)

**Goal:** A function `generate_coupons(prefix, count)` that returns unique random coupons using parallel processing.

---

### Step 2.1 — Define the public interface (skeleton only)

Open `src/generator.rs` and add the following. **Do not implement the function body yet** — just the signature, types, and constants.

```rust
use dashmap::DashSet;
use rayon::prelude::*;
use rand::Rng;

/// Every coupon code is exactly this many characters.
pub const COUPON_LENGTH: usize = 10;

/// The pool of allowed characters: lowercase a-z and digits 0-9.
const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

/// If we've tried this many times the batch size and still don't have
/// enough unique coupons, something is wrong (prefix too long, etc).
const MAX_ATTEMPTS_MULTIPLIER: usize = 20;

/// Errors that can occur during generation.
#[derive(Debug, PartialEq)]
pub enum GeneratorError {
    /// The prefix is so long there's no room for random characters.
    PrefixTooLong,
    /// The caller asked for zero coupons.
    ZeroCount,
    /// Couldn't generate enough unique coupons within the attempt limit.
    MaxAttemptsExceeded,
}

/// Generates `count` unique coupon codes with the given `prefix`.
///
/// - Total coupon length is always `COUPON_LENGTH` (10).
/// - The prefix is converted to lowercase.
/// - The random portion uses characters from `CHARSET` (a-z, 0-9).
/// - Generation is parallelized using `rayon`.
/// - Uniqueness is enforced using a concurrent `DashSet`.
///
/// # Errors
/// - `PrefixTooLong` if `prefix.len() >= COUPON_LENGTH`
/// - `ZeroCount` if `count == 0`
/// - `MaxAttemptsExceeded` if unique generation fails (extremely unlikely)
pub fn generate_coupons(
    prefix: &str,
    count: usize,
) -> Result<Vec<String>, GeneratorError> {
    todo!("Implement after writing tests")
}
```

Run `cargo build` to confirm it compiles (the `todo!()` is fine — it only panics at runtime).

---

### Step 2.2 — Write the tests (RED)

Open `tests/generator_tests.rs` and write every test before implementing anything.

```rust
use coupon_generator::generator::{generate_coupons, GeneratorError, COUPON_LENGTH};
use std::collections::HashSet;

// ───────────────────────────────────────────────
// HAPPY PATH TESTS
// ───────────────────────────────────────────────

#[test]
fn returns_the_exact_number_of_coupons_requested() {
    let coupons = generate_coupons("SAN", 100).unwrap();
    assert_eq!(coupons.len(), 100);
}

#[test]
fn every_coupon_has_exactly_10_characters() {
    let coupons = generate_coupons("SAN", 200).unwrap();
    for coupon in &coupons {
        assert_eq!(
            coupon.len(),
            COUPON_LENGTH,
            "Coupon '{}' has wrong length: {}",
            coupon,
            coupon.len()
        );
    }
}

#[test]
fn every_coupon_starts_with_the_prefix_in_lowercase() {
    let coupons = generate_coupons("ABC", 50).unwrap();
    for coupon in &coupons {
        assert!(
            coupon.starts_with("abc"),
            "Coupon '{}' does not start with 'abc'",
            coupon
        );
    }
}

#[test]
fn all_coupons_in_a_batch_are_unique() {
    let coupons = generate_coupons("SAN", 5_000).unwrap();
    let unique: HashSet<&String> = coupons.iter().collect();
    assert_eq!(
        unique.len(),
        coupons.len(),
        "Found {} duplicates in {} coupons",
        coupons.len() - unique.len(),
        coupons.len()
    );
}

#[test]
fn coupons_contain_only_lowercase_letters_and_digits() {
    let coupons = generate_coupons("SAN", 500).unwrap();
    for coupon in &coupons {
        for ch in coupon.chars() {
            assert!(
                ch.is_ascii_lowercase() || ch.is_ascii_digit(),
                "Coupon '{}' contains invalid character '{}'",
                coupon,
                ch
            );
        }
    }
}

#[test]
fn prefix_is_treated_as_case_insensitive() {
    // "SAN", "san", "San" should all produce coupons starting with "san"
    for prefix in &["SAN", "san", "San", "sAn"] {
        let coupons = generate_coupons(prefix, 10).unwrap();
        for coupon in &coupons {
            assert!(
                coupon.starts_with("san"),
                "Prefix '{}' produced coupon '{}' — expected 'san' prefix",
                prefix,
                coupon
            );
        }
    }
}

#[test]
fn empty_prefix_generates_fully_random_coupons() {
    let coupons = generate_coupons("", 100).unwrap();
    assert_eq!(coupons.len(), 100);
    for coupon in &coupons {
        assert_eq!(coupon.len(), COUPON_LENGTH);
    }
}

#[test]
fn single_character_prefix_works() {
    let coupons = generate_coupons("X", 50).unwrap();
    for coupon in &coupons {
        assert!(coupon.starts_with("x"));
        assert_eq!(coupon.len(), COUPON_LENGTH);
    }
}

#[test]
fn nine_character_prefix_leaves_one_random_character() {
    // Prefix "ABCDEFGHI" = 9 chars, leaving 1 random char.
    // Only 36 possible unique coupons (a-z, 0-9).
    let coupons = generate_coupons("ABCDEFGHI", 30).unwrap();
    assert_eq!(coupons.len(), 30);
    for coupon in &coupons {
        assert!(coupon.starts_with("abcdefghi"));
        assert_eq!(coupon.len(), COUPON_LENGTH);
    }
}

// ───────────────────────────────────────────────
// ERROR CASE TESTS
// ───────────────────────────────────────────────

#[test]
fn prefix_of_10_chars_returns_prefix_too_long_error() {
    // 10-char prefix leaves 0 random chars — impossible
    let result = generate_coupons("ABCDEFGHIJ", 10);
    assert_eq!(result, Err(GeneratorError::PrefixTooLong));
}

#[test]
fn prefix_longer_than_10_chars_returns_prefix_too_long_error() {
    let result = generate_coupons("ABCDEFGHIJKLM", 10);
    assert_eq!(result, Err(GeneratorError::PrefixTooLong));
}

#[test]
fn zero_count_returns_zero_count_error() {
    let result = generate_coupons("SAN", 0);
    assert_eq!(result, Err(GeneratorError::ZeroCount));
}

// ───────────────────────────────────────────────
// PERFORMANCE TEST
// ───────────────────────────────────────────────

#[test]
fn generates_50000_unique_coupons_without_crashing() {
    let coupons = generate_coupons("X", 50_000).unwrap();
    assert_eq!(coupons.len(), 50_000);
    let unique: HashSet<&String> = coupons.iter().collect();
    assert_eq!(unique.len(), 50_000);
}
```

### Now run the tests

```bash
cargo test
```

**Every test should FAIL** (because of `todo!()`). This is the **RED** step. You should see errors like `thread panicked at 'not yet implemented'`. That's expected and correct.

> **Commit this:**
> ```bash
> git add .
> git commit -m "Phase 2 RED: generator tests written, all failing"
> ```

---

### Step 2.3 — Implement the function (GREEN)

Now open `src/generator.rs` and replace the `todo!()` with the real implementation:

```rust
use dashmap::DashSet;
use rayon::prelude::*;
use rand::Rng;

pub const COUPON_LENGTH: usize = 10;
const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
const MAX_ATTEMPTS_MULTIPLIER: usize = 20;

#[derive(Debug, PartialEq)]
pub enum GeneratorError {
    PrefixTooLong,
    ZeroCount,
    MaxAttemptsExceeded,
}

pub fn generate_coupons(
    prefix: &str,
    count: usize,
) -> Result<Vec<String>, GeneratorError> {
    // ── Validation ──────────────────────────────────────────
    let prefix_lower = prefix.to_ascii_lowercase();

    if prefix_lower.len() >= COUPON_LENGTH {
        return Err(GeneratorError::PrefixTooLong);
    }
    if count == 0 {
        return Err(GeneratorError::ZeroCount);
    }

    // ── Setup ───────────────────────────────────────────────
    let random_len = COUPON_LENGTH - prefix_lower.len();
    let set: DashSet<String> = DashSet::new();
    let max_total_attempts = count * MAX_ATTEMPTS_MULTIPLIER;
    let batch_size = count.max(1_024);
    let mut total_attempts: usize = 0;

    // ── Parallel Generation Loop ────────────────────────────
    //
    // HOW THIS WORKS:
    //
    // 1. We create a parallel iterator over a range (0..batch_size)
    //    using rayon's `into_par_iter()`. Rayon automatically splits
    //    this across CPU cores.
    //
    // 2. Each parallel task generates ONE random coupon and tries
    //    to insert it into a `DashSet` — a thread-safe hash set.
    //    If the coupon already exists, the insert is a no-op.
    //
    // 3. We loop in batches until the set has enough unique coupons
    //    or we exceed the safety limit.
    //
    while set.len() < count {
        if total_attempts > max_total_attempts {
            return Err(GeneratorError::MaxAttemptsExceeded);
        }

        let needed = count - set.len();

        // Generate candidates in parallel.
        // We generate 2x what we need to account for duplicates.
        (0..batch_size.min(needed * 2))
            .into_par_iter()
            .for_each(|_| {
                // Early exit: stop generating if we already have enough.
                if set.len() >= count {
                    return;
                }

                // Each thread gets its own random number generator.
                // This avoids lock contention — no shared RNG.
                let mut rng = rand::thread_rng();

                // Build the random part character by character.
                let random_part: String = (0..random_len)
                    .map(|_| {
                        let idx = rng.gen_range(0..CHARSET.len());
                        CHARSET[idx] as char
                    })
                    .collect();

                let coupon = format!("{}{}", prefix_lower, random_part);

                // DashSet::insert returns false if the value
                // already existed — uniqueness is guaranteed.
                set.insert(coupon);
            });

        total_attempts += batch_size;
    }

    // ── Collect Results ─────────────────────────────────────
    // Take exactly `count` coupons from the set.
    let result: Vec<String> = set.into_iter().take(count).collect();
    Ok(result)
}
```

### Run the tests again

```bash
cargo test
```

**Every test should now PASS.** This is the **GREEN** step.

> **Commit:**
> ```bash
> git add .
> git commit -m "Phase 2 GREEN: generator implementation, all tests passing"
> ```

---

### Step 2.4 — Refactor

Read through your code. Ask yourself:

- Are variable names clear?
- Is there any duplication?
- Are the comments helpful, not obvious?

If you change anything, run `cargo test` again to make sure nothing broke. Commit after refactoring.

---

## 8. Phase 3 — Excel Exporter Module (TDD)

**Goal:** A function `export_to_excel(coupons, output_dir, base_name)` that writes coupons to `.xlsx` files, splitting into files of 10,000 rows max.

---

### Step 3.1 — Define the public interface (skeleton)

Open `src/exporter.rs`:

```rust
use rust_xlsxwriter::*;
use std::path::Path;

/// Maximum number of coupons per Excel file.
pub const MAX_PER_FILE: usize = 10_000;

/// Errors that can occur during export.
#[derive(Debug)]
pub enum ExportError {
    /// File system error (permissions, disk full, etc).
    IoError(String),
    /// Error writing Excel content.
    XlsxError(String),
}

/// Exports coupons to one or more Excel files.
///
/// - Each file contains at most `MAX_PER_FILE` (10,000) coupons.
/// - File naming: `{base_name}.xlsx`, `{base_name}_part2.xlsx`, `{base_name}_part3.xlsx`, ...
/// - Each file has a header row ("Coupon Code") followed by one coupon per row.
///
/// Returns a list of file paths that were created.
///
/// # Arguments
/// - `coupons` — the full list of generated coupons
/// - `output_dir` — the folder where files will be saved
/// - `base_name` — the base file name (without extension)
pub fn export_to_excel(
    coupons: &[String],
    output_dir: &Path,
    base_name: &str,
) -> Result<Vec<String>, ExportError> {
    todo!("Implement after writing tests")
}
```

---

### Step 3.2 — Write the tests (RED)

Open `tests/exporter_tests.rs`:

```rust
use coupon_generator::exporter::{export_to_excel, MAX_PER_FILE};
use std::path::Path;
use tempfile::tempdir;

// Helper: creates a vector of dummy coupon strings.
fn dummy_coupons(count: usize) -> Vec<String> {
    (0..count).map(|i| format!("san{:07}", i)).collect()
}

// ───────────────────────────────────────────────
// FILE COUNT TESTS
// ───────────────────────────────────────────────

#[test]
fn small_batch_produces_one_file() {
    let coupons = dummy_coupons(100);
    let dir = tempdir().unwrap();
    let files = export_to_excel(&coupons, dir.path(), "coupons").unwrap();
    assert_eq!(files.len(), 1);
}

#[test]
fn exactly_10000_coupons_produces_one_file() {
    let coupons = dummy_coupons(10_000);
    let dir = tempdir().unwrap();
    let files = export_to_excel(&coupons, dir.path(), "coupons").unwrap();
    assert_eq!(files.len(), 1);
}

#[test]
fn just_over_10000_produces_two_files() {
    let coupons = dummy_coupons(10_001);
    let dir = tempdir().unwrap();
    let files = export_to_excel(&coupons, dir.path(), "coupons").unwrap();
    assert_eq!(files.len(), 2);
}

#[test]
fn large_batch_splits_correctly() {
    let coupons = dummy_coupons(25_000);
    let dir = tempdir().unwrap();
    let files = export_to_excel(&coupons, dir.path(), "coupons").unwrap();
    // 25,000 / 10,000 = 2 full files + 1 partial = 3 files
    assert_eq!(files.len(), 3);
}

#[test]
fn empty_list_produces_one_file() {
    let coupons: Vec<String> = vec![];
    let dir = tempdir().unwrap();
    let files = export_to_excel(&coupons, dir.path(), "coupons").unwrap();
    assert_eq!(files.len(), 1);
}

// ───────────────────────────────────────────────
// FILE EXISTENCE TESTS
// ───────────────────────────────────────────────

#[test]
fn all_returned_file_paths_exist_on_disk() {
    let coupons = dummy_coupons(25_000);
    let dir = tempdir().unwrap();
    let files = export_to_excel(&coupons, dir.path(), "coupons").unwrap();
    for file_path in &files {
        assert!(
            Path::new(file_path).exists(),
            "File does not exist: {}",
            file_path
        );
    }
}

// ───────────────────────────────────────────────
// FILE NAMING TESTS
// ───────────────────────────────────────────────

#[test]
fn first_file_uses_base_name_directly() {
    let coupons = dummy_coupons(100);
    let dir = tempdir().unwrap();
    let files = export_to_excel(&coupons, dir.path(), "my_coupons").unwrap();
    let file_name = Path::new(&files[0])
        .file_name()
        .unwrap()
        .to_string_lossy();
    assert_eq!(file_name, "my_coupons.xlsx");
}

#[test]
fn subsequent_files_have_part_numbers() {
    let coupons = dummy_coupons(25_000);
    let dir = tempdir().unwrap();
    let files = export_to_excel(&coupons, dir.path(), "export").unwrap();

    let names: Vec<String> = files
        .iter()
        .map(|f| {
            Path::new(f)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect();

    assert_eq!(names[0], "export.xlsx");
    assert_eq!(names[1], "export_part2.xlsx");
    assert_eq!(names[2], "export_part3.xlsx");
}

// ───────────────────────────────────────────────
// FILE SIZE SANITY CHECK
// ───────────────────────────────────────────────

#[test]
fn generated_files_are_not_empty() {
    let coupons = dummy_coupons(100);
    let dir = tempdir().unwrap();
    let files = export_to_excel(&coupons, dir.path(), "coupons").unwrap();
    for file_path in &files {
        let metadata = std::fs::metadata(file_path).unwrap();
        assert!(
            metadata.len() > 0,
            "File is empty: {}",
            file_path
        );
    }
}
```

### Run the tests

```bash
cargo test
```

All new tests should **FAIL**. That's RED. Commit.

---

### Step 3.3 — Implement the function (GREEN)

Open `src/exporter.rs` and replace the `todo!()`:

```rust
use rust_xlsxwriter::*;
use std::path::Path;

pub const MAX_PER_FILE: usize = 10_000;

#[derive(Debug)]
pub enum ExportError {
    IoError(String),
    XlsxError(String),
}

pub fn export_to_excel(
    coupons: &[String],
    output_dir: &Path,
    base_name: &str,
) -> Result<Vec<String>, ExportError> {
    // ── Split into chunks ───────────────────────────────────
    // If the list is empty, we still produce one file (with just a header).
    // Otherwise, chunk into groups of MAX_PER_FILE.
    let chunks: Vec<&[String]> = if coupons.is_empty() {
        vec![&[]]
    } else {
        coupons.chunks(MAX_PER_FILE).collect()
    };

    let mut created_files: Vec<String> = Vec::new();

    for (index, chunk) in chunks.iter().enumerate() {
        // ── Build file name ─────────────────────────────────
        let file_name = if index == 0 {
            format!("{}.xlsx", base_name)
        } else {
            format!("{}_part{}.xlsx", base_name, index + 1)
        };

        let full_path = output_dir.join(&file_name);

        // ── Create workbook and worksheet ───────────────────
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        // Write header row
        worksheet
            .write_string(0, 0, "Coupon Code")
            .map_err(|e| ExportError::XlsxError(e.to_string()))?;

        // Write each coupon on its own row (row 1, 2, 3, ...)
        for (row, coupon) in chunk.iter().enumerate() {
            worksheet
                .write_string((row + 1) as u32, 0, coupon)
                .map_err(|e| ExportError::XlsxError(e.to_string()))?;
        }

        // ── Save to disk ────────────────────────────────────
        workbook
            .save(&full_path)
            .map_err(|e| ExportError::IoError(e.to_string()))?;

        created_files.push(full_path.to_string_lossy().to_string());
    }

    Ok(created_files)
}
```

### Run the tests

```bash
cargo test
```

**All tests should pass.** Commit.

---

## 9. Phase 4 — GUI Module

**Goal:** A simple window with two input fields and a button.

### Step 4.1 — Build the UI

Open `src/ui.rs`:

```rust
use eframe::egui;
use std::path::PathBuf;

use crate::exporter::export_to_excel;
use crate::generator::generate_coupons;

/// The main application state.
pub struct CouponApp {
    prefix: String,
    count_input: String,
    status_message: String,
}

impl Default for CouponApp {
    fn default() -> Self {
        Self {
            prefix: String::new(),
            count_input: "100".to_string(),
            status_message: String::new(),
        }
    }
}

impl eframe::App for CouponApp {
    /// Called every frame. Draws the UI and handles interaction.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // ── Title ───────────────────────────────────────
            ui.heading("Coupon Generator");
            ui.add_space(12.0);

            // ── Prefix input ────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Prefix:");
                ui.text_edit_singleline(&mut self.prefix);
            });

            // ── Count input ─────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Number of coupons:");
                ui.text_edit_singleline(&mut self.count_input);
            });

            ui.add_space(12.0);

            // ── Generate button ─────────────────────────────
            if ui.button("Generate & Export to Excel").clicked() {
                self.handle_generate();
            }

            ui.add_space(12.0);

            // ── Status message ──────────────────────────────
            if !self.status_message.is_empty() {
                ui.label(&self.status_message);
            }
        });
    }
}

impl CouponApp {
    /// Validates input, generates coupons, exports to Excel.
    fn handle_generate(&mut self) {
        // Parse the count
        let count: usize = match self.count_input.trim().parse() {
            Ok(n) if n > 0 => n,
            _ => {
                self.status_message =
                    "Please enter a valid positive number.".to_string();
                return;
            }
        };

        // Generate
        self.status_message = "Generating coupons...".to_string();

        match generate_coupons(&self.prefix, count) {
            Ok(coupons) => {
                // Determine output directory (Desktop, or current dir as fallback)
                let output_dir = dirs::desktop_dir()
                    .unwrap_or_else(|| PathBuf::from("."));

                // Export
                match export_to_excel(&coupons, &output_dir, "coupons") {
                    Ok(files) => {
                        self.status_message = format!(
                            "Done! {} coupons saved to {} file(s) on your Desktop.",
                            coupons.len(),
                            files.len()
                        );
                    }
                    Err(e) => {
                        self.status_message =
                            format!("Export failed: {:?}", e);
                    }
                }
            }
            Err(e) => {
                self.status_message =
                    format!("Generation failed: {:?}", e);
            }
        }
    }
}
```

### Step 4.2 — Wire up `main.rs`

Replace `src/main.rs`:

```rust
pub mod generator;
pub mod exporter;
mod ui;

use ui::CouponApp;

fn main() -> eframe::Result<()> {
    // Configure the native window
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([420.0, 260.0])    // width x height in pixels
            .with_resizable(true),
        ..Default::default()
    };

    // Launch the app
    eframe::run_native(
        "Coupon Generator",
        options,
        Box::new(|_cc| Ok(Box::new(CouponApp::default()))),
    )
}
```

### Step 4.3 — Test it manually

```bash
cargo run
```

A window should appear. Try entering a prefix and a count, click the button, and check your Desktop for the Excel files. Commit.

---

## 10. Phase 5 — Integration & Manual Testing

Before building the final binary, run through this checklist manually:

| # | Test | Expected Result | Pass? |
|---|------|-----------------|-------|
| 1 | Prefix "SAN", count 10 | 1 Excel file on Desktop with 10 rows + header |  |
| 2 | Prefix "SAN", count 10,000 | 1 Excel file with 10,000 rows + header |  |
| 3 | Prefix "SAN", count 10,001 | 2 Excel files (10,000 + 1) |  |
| 4 | Prefix "SAN", count 25,000 | 3 Excel files (10k + 10k + 5k) |  |
| 5 | Prefix "", count 100 | 1 file, all coupons are 10 random chars |  |
| 6 | Prefix "ABCDEFGHIJ", count 10 | Error message shown in UI |  |
| 7 | Count "0" | Error message shown in UI |  |
| 8 | Count "abc" | "Please enter a valid number" shown in UI |  |
| 9 | Open any exported Excel in Excel/LibreOffice | Column A header is "Coupon Code", data below looks correct |  |
| 10 | Verify no duplicate coupons across all files | Use Excel's COUNTIF or sort & scan. All unique. |  |

Also confirm all automated tests still pass:

```bash
cargo test
```

---

## 11. Phase 6 — Build for Distribution

### Windows

```bash
cargo build --release
```

The output is `target/release/coupon-generator.exe`. This single file can be copied to any Windows machine and double-clicked to run. No installation needed.

### macOS

```bash
cargo build --release
```

The output is `target/release/coupon-generator` (a binary). To create a proper `.app` bundle:

```bash
cargo install cargo-bundle    # one-time setup
```

Add this to `Cargo.toml`:

```toml
[package.metadata.bundle]
name = "Coupon Generator"
identifier = "com.yourcompany.coupongenerator"
icon = ["icon.png"]          # optional: add a 256x256 icon
```

Then run:

```bash
cargo bundle --release
```

This creates `target/release/bundle/osx/Coupon Generator.app`. You can drag this into the Applications folder or distribute it directly.

---

## Appendix A — Glossary

| Term | Meaning |
|------|---------|
| **Crate** | A Rust package/library. Like "npm package" in JavaScript or "gem" in Ruby. |
| **`cargo`** | Rust's build tool and package manager. Compiles code, runs tests, manages dependencies. |
| **`cargo test`** | Runs all unit and integration tests in the project. |
| **`cargo build --release`** | Compiles optimized production binary (slow to compile, fast to run). |
| **`Vec<String>`** | A growable list (vector) of text strings. |
| **`Result<T, E>`** | Rust's way of saying "this function returns either a success value `T` or an error `E`." |
| **`unwrap()`** | Extracts the success value from a Result. Panics if it's an error. Fine in tests, avoid in production code. |
| **`DashSet`** | A hash set that multiple threads can read/write simultaneously without crashing. |
| **`into_par_iter()`** | Rayon method that turns a normal iterator into a parallel one across CPU cores. |
| **`todo!()`** | A placeholder macro. Compiles fine but panics at runtime. Used during TDD to mark unimplemented code. |
| **TDD** | Test-Driven Development. Write the test first, watch it fail, then write the code. |

---

## Appendix B — Troubleshooting

### "error[E0432]: unresolved import"

You probably forgot to add `pub` to the module declaration in `main.rs`. Make sure you have:

```rust
pub mod generator;
pub mod exporter;
```

### "thread panicked at 'not yet implemented'"

You're running code that still has `todo!()` in it. This is expected during the RED phase of TDD. Implement the function body.

### Tests pass but the app doesn't launch

Make sure `eframe` is in your `[dependencies]` (not `[dev-dependencies]`). Dev dependencies are only available during testing.

### Excel file opens but looks empty

Check that you're writing to row `(row + 1) as u32`, not row `0`. Row 0 is the header.

### "cargo bundle" is not found

Install it with `cargo install cargo-bundle`. It's a separate tool, not built into Cargo.

### Compilation is slow

Normal for the first build (downloads and compiles all dependencies). Subsequent builds are fast. Release builds (`--release`) are always slower to compile than debug builds.
