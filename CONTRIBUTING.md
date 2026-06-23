# Contributing to S.P.E.C.U.L.A.M. Solver

Thank you for your interest in contributing to S.P.E.C.U.L.A.M. Solver! We welcome community contributions to make this optimizer faster, safer, and more feature-rich for low-latency DeFi.

---

## 🛠️ How to Contribute

### 1. Report Issues
If you find a bug or have a feature request, please open an issue on the GitHub repository. Provide:
- A clear description of the problem.
- A minimal reproducible example (if applicable).
- Expected vs. actual behavior.

### 2. Submit Pull Requests (PRs)
To make code contributions:
1. **Fork** the repository and create a new branch from `main`:
   ```bash
   git checkout -b feature/my-amazing-feature
   ```
2. **Make your changes**. Ensure your code is clean, documented, and follows Rust best practices.
3. **Format your code** using standard cargo tools:
   ```bash
   cargo fmt
   ```
4. **Lint your code** and fix any warnings or recommendations:
   ```bash
   cargo clippy --all-targets -- -D warnings
   ```
5. **Run the test suite** to ensure everything passes:
   ```bash
   cargo test
   ```
6. **Submit the PR** with a clear title and description explaining the changes, their rationale, and how they were tested.

---

## 📝 Code of Conduct
Please be respectful and constructive in all communication, including issues, pull requests, and discussions.

---

## 📄 License
By contributing to this repository, you agree that your contributions will be licensed under the MIT License.
