# Sudoku 402

Sudoku 401 的 Web 克隆，使用 Rust、Leptos 和 WebAssembly 构建，适配 iPhone 浏览器。

## 构建与运行

安装 Rust、WebAssembly 编译目标和 Trunk：

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --version 0.21.14 --locked
```

在项目目录运行：

```sh
trunk serve             # 本地开发
trunk build --release   # 构建到 dist/
```

## 测试

```sh
cargo test --workspace
```

