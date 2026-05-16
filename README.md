# 🚀 RustBasic CLI

**RustBasic CLI** is the high-performance command-line interface for the [RustBasic Framework](https://github.com/herisvan321/rustbasic). It is designed to be blazing fast, modular, and extremely easy to use.

## ✨ Features

- **On-Demand Runtime**: Utility commands start in **~0.008s** by bypassing the async runtime when not needed.
- **Magic Scaffolding**: Generate models, controllers, and migrations with a single command.
- **Seamless Delegation**: Run database migrations and seeders that automatically use your project's local code.
- **Build Manager**: Cross-compile your project for Windows, Linux, and macOS with ease.
- **Security Checkup**: Automated security audits for your RustBasic applications.

## 📦 Installation

To install the CLI globally on your system:

```bash
cargo install rustbasic-cli
```

*Note: Requires Rust and Cargo to be installed. If you prefer building from source:*

```bash
git clone https://github.com/herisvan321/rustbasic-cli
cd rustbasic-cli
cargo install --path .
```

## 🛠️ Usage

### Project Management
- `rustbasic new <project_name>`: Create a new RustBasic project from the official template.
- `rustbasic serve`: Start the development server with auto-reload (using `cargo-watch`).

### Scaffolding (Generators)
- `rustbasic make:controller <Name>`: Create a new controller.
- `rustbasic make:model <Name>`: Create a new model and its corresponding migration.
- `rustbasic make:migration <name>`: Create a new database migration file (standard/create).
- `rustbasic make:migration:add <column> <table>`: Create a new migration to add a column.
- `rustbasic make:auth`: Scaffold a complete authentication system (Login, Register, Dashboard).

### Database & Storage
- `rustbasic migrate`: Run pending migrations.
- `rustbasic migrate:refresh`: Rollback and re-run all migrations.
- `rustbasic migrate:rollback`: Rollback the last migration batch.
- `rustbasic db:seed`: Seed the database with initial data.
- `rustbasic storage:link`: Create a symbolic link from `public/storage` to `storage/app/public` to make uploaded files accessible via URL.

### Utilities
- `rustbasic key:generate`: Generate a new `APP_KEY` for your `.env` file.
- `rustbasic route:list`: List all registered routes in your application.
- `rustbasic check:security`: Run a security health check on your project.
- `rustbasic build`: Interactive project build manager.
- `rustbasic version`: Display the current CLI version.

## 🤝 Contributing

Contributions are welcome! Feel free to open issues or submit pull requests to improve the CLI.

## 📄 License

This project is licensed under the MIT License.
