# template-teleporter

## CI/CD Workflows

This repository includes the following CI/CD workflows:

1. **Linting**:
   - Runs `cargo check`, `cargo fmt`, `cargo clippy`, and `cargo deny` to ensure code quality and compliance.
   - Each task is executed in its own job for better isolation and parallelism.
   - Triggered on every push to any branch, pull request, and manually via `workflow_dispatch`.

2. **CI**:
   - Executes `cargo test --doc --all-features` to run documentation tests.
   - Runs unit tests with coverage using `cargo llvm-cov`.
   - Uploads coverage data to Codecov using the latest Codecov action.
   - Triggered on every push to any branch, pull request, and manually via `workflow_dispatch`.

3. **Mutation Testing**:
   - Runs `cargo mutants` to perform mutation testing.
   - Triggered manually via `workflow_dispatch`.

Refer to the `.github/workflows/` directory for workflow definitions.
