---
title: "[FEATURE]: System 3 Belief Graph and Overseer"
labels: "enhancement, ai-plan, jules"
assignees: ""
---

## 💡 Feature Description
Implement the core innovation of AgentRAG: System 3 (Meta-Cognitive Overseer) and the Belief Graph in SurrealDB.

## 🎯 Problem it Solves
Prevents LLM hallucinations by cross-referencing drafted answers against a formal graph database of validated facts.

## 📐 Proposed Solution
- Implement `src/memory/belief_graph.rs` to insert and query Graph Nodes in SurrealDB.
- Implement `src/agents/system3.rs` (The Overseer). If System 3 detects a contradiction in System 2's draft against the Belief Graph, it must veto the response and send feedback.

## 📝 Notes for AI Agent (Jules & Copilot)
> **Jules Instructions:**
> 1. Read `.gitcore/ARCHITECTURE.md` and `AGENTS.md` before proceeding.
> 2. Strictly follow the Git-Core Protocol (do not track state in `TODO.md` or any unauthorized file).
> 3. Execute the changes and ensure you run `./scripts/ai-report.ps1` before finishing the task.

The loop between System 2 and System 3 should be orchestrated by `src/agents/supervisor.rs`.
