# TLS Notary Python Wrapper

This package provides a Python wrapper for TLS Notary, enabling Python applications to use TLS Notary functions by
exposing Rust-based functions as Python bindings through `pyo3`.

## Requirements

- Python 3.6+
- Rust and `maturin` for building and developing the package

## Setup and Installation

1. **Create and Activate a Python Virtual Environment**:
   ```bash
   python -m venv .env
   source .env/bin/activate
   ```

2. **Install Maturin**:
   Maturin is required to compile Rust code into Python bindings.
   ```bash
   pip install maturin
   ```

3. **Initialize the Project with Maturin**:
   Run the following command to set up the project with the necessary PyO3 bindings:
   ```bash
   maturin init --bindings pyo3
   ```

4. **Develop and Build the Package**:
   Use Maturin to develop the package in your virtual environment.
   ```bash
   maturin develop
   ```

   This command will build and install the package within the virtual environment, allowing Python to access the
   Rust-compiled bindings.

## Testing the Package

A sample script, `sample.py`, is provided to test the package functionality. To run this script, ensure the virtual
environment is activated and then execute:

```bash
python sample.py
```

This script demonstrates how to use the Python wrapper for TLS Notary functions provided by the package.

## Important Notes

- **Compilation with Cargo**: By default, the package will not compile with `cargo build` due to the exposed Python
  bindings. Instead, use `maturin develop` as shown above to build the package with the correct bindings for Python.