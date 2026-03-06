---
title: "[FEATURE]: System 1 and 2 Implementation"
labels: "enhancement, ai-plan, jules"
assignees: ""
---

## 💡 Feature Description
Implement the foundation of the AgentRAG swarm: the Retriever (System 1) and Reasoner (System 2).
System 1 should query SurrealDB for vector similarities.
System 2 should take that context and use Chain of Thought to draft an answer.

## 🎯 Problem it Solves
Lays the groundwork for the RAG agent swarm before introducing meta-cognition.

## 📐 Proposed Solution
- Implement `src/agents/system1.rs` using `adk-rust` tools for SurrealDB vector search.
- Implement `src/agents/system2.rs` using a CoT prompt template.

## 📝 Notes for AI Agent (Jules & Copilot)
> **Jules Instructions:**
> 1. Read `.gitcore/ARCHITECTURE.md` and `AGENTS.md` before proceeding.
> 2. Strictly follow the Git-Core Protocol (do not track state in `TODO.md` or any unauthorized file).
> 3. Execute the changes and ensure you run `./scripts/ai-report.ps1` before finishing the task.

Ensure `tokio` is used for all async calls and `zavora-ai/adk-rust` correctly formats the LLM agnostic calls.
