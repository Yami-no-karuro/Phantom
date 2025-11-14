# Phantom

## Phantom proxy - A Rust MVP project

### Intro

The basic idea is to have some sort of **middleware** to handle security checks before the application layer.  
Basically, every request is forced through an **additional step** before even reaching whatever app or framework is forwarded to.

This approach allows to perform security verifications and controls more efficiently, without affecting the next layer at all.

This **MVP**, in a very simple way, focuses on checking whether the current request path is found within a list of paths that are **typically used by automatic scanning tools** such as [Nikto](https://www.kali.org/tools/nikto/) or [Burpsuite](https://www.kali.org/tools/burpsuite/).  

### Important Note

This project is intended as a **minimum viable product** (MVP) and is **not** designed for **production** use.  
It serves to illustrate the basic functionality of a security-focused **proxy middleware** but lacks the robustness, security measures, and extensive testing necessary for a **production environment**.

### Features & limitations

- **Basic filtering**: Only the paths specified in the predefined list will be checked.
- **Fast deployment**: Quick setup to demonstrate the architecture without complex configurations.
- **Extensibility**: While this MVP is simple, it can be expanded with additional security checks and features for further development.

### Gettings Started

- Make sure to have the [Rust](https://rust-lang.org/) programming language installed on your machine.
- Clone the [Repository](https://github.com/Yami-no-karuro/Phantom.git).
- Build and run the project via `cargo run <proxy> <forward-to>`.
- Navigate to `http://localhost:<forward-to>/` on your browser.
