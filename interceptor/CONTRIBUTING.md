<div align="center">

![Interceptor Banner](interceptor/assets/banner.png)

# Contributing to Interceptor

</div>

Thank you for your interest in contributing to Interceptor! This document outlines the process and guidelines for contributing to this **proprietary** project.

---

## 🔒 **Important: Proprietary License**

**Please note**: Interceptor is licensed under a **proprietary license**, not an open-source license. This means:

-   ✅ You CAN view the source code
-   ✅ You CAN suggest improvements
-   ✅ You CAN report bugs
-   ✅ You CAN submit pull requests
-   ❌ You CANNOT fork for your own projects
-   ❌ You CANNOT redistribute the code
-   ❌ You CANNOT use it commercially without a license

**By contributing, you agree that**:

1. All contributions become the property of S1BGr0uP
2. You transfer all rights to your contributions to S1BGr0uP
3. S1BGr0uP may use your contributions in any way, including commercial products
4. You will be credited in the project documentation

---

## 📋 **Table of Contents**

1. [Code of Conduct](#code-of-conduct)
2. [How to Contribute](#how-to-contribute)
3. [Reporting Bugs](#reporting-bugs)
4. [Suggesting Features](#suggesting-features)
5. [Pull Request Process](#pull-request-process)
6. [Development Setup](#development-setup)
7. [Coding Standards](#coding-standards)
8. [Contributor Recognition](#contributor-recognition)

---

## 📜 **Code of Conduct**

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors.

### Our Standards

-   ✅ Be respectful and professional
-   ✅ Accept constructive criticism gracefully
-   ✅ Focus on what's best for the project
-   ❌ No harassment, discrimination, or trolling
-   ❌ No spam or self-promotion

### Enforcement

Violations may result in:

-   Warning
-   Temporary ban from contributing
-   Permanent ban from the project

Report violations to: s1bgr0up.root@gmail.com

---

## 🤝 **How to Contribute**

### Ways to Contribute

1. **Report Bugs** - Help us identify issues
2. **Suggest Features** - Share your ideas
3. **Submit Pull Requests** - Fix bugs or add features
4. **Improve Documentation** - Help others understand the project
5. **Answer Questions** - Help other users in discussions

### What We're Looking For

-   🐛 Bug fixes
-   📝 Documentation improvements
-   🎨 UI/UX enhancements
-   ⚡ Performance optimizations
-   🧪 Test coverage improvements
-   🌍 Translations (future)

### What We're NOT Looking For

-   ❌ Major architectural changes (discuss first)
-   ❌ New dependencies without justification
-   ❌ Features that don't align with project goals
-   ❌ Code that violates our license

---

## 🐛 **Reporting Bugs**

### Before Reporting

1. Check existing issues
2. Verify you're using the latest version
3. Ensure it's not a configuration issue

### Bug Report Template

```markdown
**Description**
Clear description of the bug

**Steps to Reproduce**

1. Step one
2. Step two
3. ...

**Expected Behavior**
What should happen

**Actual Behavior**
What actually happens

**Environment**

-   OS: [e.g., Ubuntu 22.04]
-   Interceptor Version: [e.g., 2.0.0]
-   Rust Version: [e.g., 1.70.0]

**Logs**
```

Paste relevant logs here

```

**Additional Context**
Any other relevant information
```

---

## 💡 **Suggesting Features**

### Before Suggesting

1. Check if it already exists
2. Check if it's already been suggested
3. Consider if it aligns with project goals

### Feature Request Template

```markdown
**Feature Description**
Clear description of the feature

**Problem Statement**
What problem does this solve?

**Proposed Solution**
How should it work?

**Alternatives Considered**
Other ways to solve this

**Use Case**
Real-world scenario where this is useful

**Additional Context**
Mockups, examples, etc.
```

---

## 🔄 **Pull Request Process**

### Before Submitting

1. **Discuss First** - For major changes, open an issue first
2. **Fork & Branch** - Create a feature branch
3. **Follow Standards** - Adhere to coding standards
4. **Test** - Ensure all tests pass
5. **Document** - Update relevant documentation

### PR Checklist

-   [ ] Code follows project style guidelines
-   [ ] Self-review completed
-   [ ] Comments added for complex code
-   [ ] Documentation updated
-   [ ] No new warnings generated
-   [ ] Tests added/updated
-   [ ] All tests pass locally
-   [ ] Commits follow conventional commits format

### PR Template

```markdown
## Description

Brief description of changes

## Motivation

Why is this change needed?

## Type of Change

-   [ ] Bug fix
-   [ ] New feature
-   [ ] Breaking change
-   [ ] Documentation update
-   [ ] Performance improvement

## Testing

How was this tested?

## Screenshots (if applicable)

Add screenshots here

## Checklist

-   [ ] Code follows style guidelines
-   [ ] Self-reviewed
-   [ ] Commented complex code
-   [ ] Updated documentation
-   [ ] No new warnings
-   [ ] Added tests
-   [ ] Tests pass
```

### Review Process

1. **Automated Checks** - CI/CD must pass
2. **Code Review** - Maintainer review
3. **Testing** - Manual testing if needed
4. **Approval** - Requires 1 maintainer approval
5. **Merge** - Squash and merge

### After Merge

-   Your contribution becomes property of S1BGr0uP
-   You'll be added to contributors list
-   Changes may be modified or removed at S1BGr0uP's discretion

---

## 🛠️ **Development Setup**

### Prerequisites

-   Rust 1.70+ ([rustup](https://rustup.rs/))
-   Node.js 18+ ([nvm](https://github.com/nvm-sh/nvm))
-   Git

### Setup Steps

```bash
# Clone the repository
git clone https://github.com/S1b-Team/int3rceptor.git
cd interceptor

# Build backend
cargo build

# Setup frontend
cd ui
npm install
npm run dev

# Run tests
cargo test
```

### Development Workflow

```bash
# Create feature branch
git checkout -b feature/my-feature

# Make changes
# ...

# Run tests
cargo test
cargo clippy

# Commit (use conventional commits)
git commit -m "feat: add new feature"

# Push
git push origin feature/my-feature

# Open PR on GitHub
```

---

## 📐 **Coding Standards**

### Rust

-   **Style**: Follow `rustfmt` defaults
-   **Linting**: Pass `clippy` without warnings
-   **Naming**: `snake_case` for functions/variables, `PascalCase` for types
-   **Comments**: Document public APIs
-   **Error Handling**: Use `Result` and `?` operator
-   **Async**: Use `tokio` for async operations

```rust
// Good
pub async fn fetch_data(url: &str) -> Result<Data, Error> {
    let response = reqwest::get(url).await?;
    Ok(response.json().await?)
}

// Bad
pub async fn fetchData(url: &str) -> Data {
    reqwest::get(url).await.unwrap().json().await.unwrap()
}
```

### TypeScript/Vue

-   **Style**: Follow project `.eslintrc`
-   **Types**: Prefer explicit types over `any`
-   **Components**: Use Composition API
-   **Naming**: `camelCase` for variables, `PascalCase` for components

```typescript
// Good
interface User {
    id: number;
    name: string;
    email: string;
}

const fetchUser = async (id: number): Promise<User> => {
    const response = await fetch(`/api/users/${id}`);
    return response.json();
};

// Bad
const fetchUser = async (id: any): Promise<any> => {
    return (await fetch(`/api/users/${id}`)).json();
};
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add WebSocket support
fix: resolve memory leak in proxy
docs: update README with examples
style: format code with rustfmt
refactor: simplify rule engine
test: add tests for intruder
chore: update dependencies
```

---

## 🏅 **Contributor Recognition**

### Hall of Fame

Contributors will be recognized in:

-   `CONTRIBUTORS.md` file
-   GitHub contributors page
-   Release notes (for significant contributions)
-   Project website (future)

### Contribution Tiers

-   🥉 **Bronze**: 1-5 merged PRs
-   🥈 **Silver**: 6-15 merged PRs
-   🥇 **Gold**: 16+ merged PRs or major feature
-   💎 **Diamond**: Exceptional long-term contributor

### Rewards (Optional)

For significant contributions, S1BGr0uP may offer:

-   Free commercial license
-   Swag (t-shirts, stickers)
-   Recognition on social media
-   Reference letter for employment

---

## 📞 **Contact**

### Questions?

-   💬 **GitHub Discussions**: https://github.com/S1b-Team/int3rceptor/discussions
-   📧 **Email**: s1bgr0up.root@gmail.com
-   💬 **Matrix**: @ind4skylivey:matrix.org

### Maintainers

-   **Lead**: @ind4skylivey
-   **Team**: S1BGr0uP Core Team

---

## 📄 **Legal**

### Contributor License Agreement (CLA)

By contributing, you agree to the following:

1. **Rights Transfer**: You grant S1BGr0uP perpetual, worldwide, non-exclusive, royalty-free, irrevocable license to use, modify, and distribute your contributions.

2. **Ownership**: S1BGr0uP becomes the owner of all contributions.

3. **Original Work**: You confirm that your contribution is your original work.

4. **No Conflicts**: Your contribution doesn't violate any third-party rights.

5. **No Warranty**: Contributions are provided "as is" without warranty.

### Why This Matters

This CLA allows S1BGr0uP to:

-   Maintain full control over the project
-   Offer commercial licenses
-   Change the license in the future if needed
-   Protect against legal issues

---

## 🙏 **Thank You!**

We appreciate your interest in improving Interceptor. While this is a proprietary project, we value community input and collaboration.

Your contributions help make Interceptor better for everyone!

---

**Last Updated**: 2025-11-21
**Version**: 2.0.1
