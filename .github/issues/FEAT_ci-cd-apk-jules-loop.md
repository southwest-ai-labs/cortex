---
title: "[FEATURE]: APK Build and CI/CD Loop for Jules"
labels: "enhancement, ai-plan, jules"
assignees: ""
---

## 💡 Feature Description
Implement `cargo-ndk` scripts to build the Rust swarms into an Android APK, alongside a GitHub Action that acts as an auto-correction feedback loop for Jules.

## 🎯 Problem it Solves
Enables deployment to mobile while ensuring the AI agent (Jules) can self-correct compilation or test errors by reading CI logs.

## 📐 Proposed Solution
- Add a script in `scripts/build_apk.sh` or `build_apk.ps1` configuring Android NDK.
- Refine `.github/workflows/jules-ci-cd-loop.yml` so that if the APK build fails or tests fail, it comments on the PR with `@jules` and the error logs.

## 📝 Notes for AI Agent (Jules & Copilot)
> **Jules Instructions:**
> 1. Read `.gitcore/ARCHITECTURE.md` and `AGENTS.md` before proceeding.
> 2. Strictly follow the Git-Core Protocol (do not track state in `TODO.md` or any unauthorized file).
> 3. Execute the changes and ensure you run `./scripts/ai-report.ps1` before finishing the task.

Make sure the workflow triggers on pull requests and uses `gh` CLI for comments.
