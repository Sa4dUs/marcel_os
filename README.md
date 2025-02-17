# **marcel_os**

<!--
![Build Status](https://img.shields.io/github/workflow/status/sa4dus/marcel_os/CI)
![Test Coverage](https://img.shields.io/codecov/c/github/sa4dus/marcel_os)
![License](https://img.shields.io/github/license/sa4dus/marcel_os)
![Version](https://img.shields.io/github/v/release/sa4dus/marcel_os)
-->

![Programming Language](https://img.shields.io/badge/language-Rust-orange)

**marcel_os** is an operating system project developed in Rust. This repository contains the source code and resources necessary to build and run.

## Features

-   **Bootable Kernel**: A fully functional kernel capable of booting on x86_64 hardware.
-   **VGA Buffer Driver**: Direct manipulation of the screen using VGA text mode.
-   **Unit and Integration Test Suite**: Comprehensive tests to ensure code reliability and stability.
-   **CPU Exception Handling**: Mechanisms to handle CPU exceptions and faults gracefully.
-   **Interrupt Handling**: Efficient management of hardware and software interrupts.
-   **Four-Level Paging**: Advanced memory management using four-level paging.
-   **Dynamic Memory Management**: Allocation and deallocation of memory at runtime.
-   **Single-Core Preemptive Executor**: A scheduler that allows preemptive multitasking on a single core.
-   **Keyboard Support**: Basic input handling for keyboard devices.
-   **Simple CLI**: A command-line interface for user interaction.

## Getting Started

### Prerequisites

-   **Rust Compiler**: Ensure you have the latest stable version of Rust installed. You can download it from [rust-lang.org](https://www.rust-lang.org/).
-   **Cargo**: Rust's package manager, included with the Rust installation.
-   **QEMU**: An emulator to run **marcel_os** without physical hardware. Download it from [qemu.org](https://www.qemu.org/).

### Installing the Nightly Version of Rust

To build and run **marcel_os**, it's recommended to use the 2024-12-31 nightly version of Rust. Follow these steps:

1. **Install the Latest Nightly Toolchain**:

    Use `rustup` to install a valid nightly version:

    ```sh
    rustup toolchain install nightly-2024-12-31
    ```

2. **Set the Toolchain for the Current Directory**:
   To use this specific nightly version in your current project directory, set an override:

    ```sh
    rustup override set nightly-2024-12-31
    ```

    This ensures that any `cargo` or `rustc` commands run within this directory use the specified nigthly version.

3. **Verify the Active Toolchain**:
   Confirm that the override is set correctly:
    ```sh
    rustup show
    ```
    Look for the "overrides" section to ensure the correct toolchain is active in your directory.

### Installing required `rustup` components

**marcel_os** requires additional Rust components for building:

1. **Add the Rust Standard Library Source**:
   This component provides the source code of the Rust standard library, which is necessary for building the kernel:
    ```sh
    rustup component add rust-src
    ```
2. **Add LLVM Tools**:
   The LLVM tools are required for certain build processes:

    ```sh
    rustup component add llvm-tools-preview
    ```

    These components are installed per toolchain, so ensure you're using the correct toolchain when adding them.

### Building **marcel_os**

1. **Clone the Repository**:

    ```sh
    git clone https://github.com/sa4dus/marcel_os.git
    cd marcel_os
    ```

2. **Build the Kernel**:

    ```sh
    cargo build --release
    ```

    This command compiles the kernel and outputs the binary in the `target/release` directory

3. **Running marcel_os**:
   After building, you can run **marcel_os** locally in your machine using QEMU:
    ```sh
    cargo run
    ```

## Contributing

Contributions are welcome! Please follow these steps:

1. **Fork the Repository**: Click on the "Fork" button at the top right of this page.
2. **Create a New branch**: Use a descriptive name for your branch.
3. **Make Changes**: Implement your feature or fix
4. **Commit Changes**: Write clear and concise commit messages.

    ```sh
        git commit -m "[prefix]: [description of the changes]
    ```

5. **Push to Your Fork**:

    ```sh
    git push origin feature-name
    ```

6. **Submit a Pull Request**: Navigate to the original repository and click on "New Pull Request".

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## References

-   **Writing an OS in Rust**: A comprehensive tutorial on building an operating system in Rust. [https://os.phil-opp.com/](https://os.phil-opp.com/)
-   **Rust OS Comparison**: A comparison of various Rust-based operating systems. [https://github.com/flosse/rust-os-comparison](https://github.com/flosse/rust-os-comparison)
-   **Operating Systems from 0 to 1**: A book that guides readers through building an operating system from scratch.
-   **Systems Programming: A Programmer's Perspective**: A book that provides insights into systems programming concepts.
-   **Redox OS**: An operating system written in Rust. [https://github.com/redox-os/redox](https://github.com/redox-os/redox)
-   **Rust OSDev**: A community focused on Rust-based operating system development. [https://github.com/rust-osdev](https://github.com/rust-osdev)
-   **OSDev Wiki - Expanded Main Page**: A comprehensive resource for operating system development. [https://wiki.osdev.org/Expanded_Main_Page](https://wiki.osdev.org/Expanded_Main_Page)
