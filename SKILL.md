# Agent Skills Standard (SKILL.md)

This file defines the Agent Skills standard for the Llama-Manager system. Agent skills are modular capabilities that extend the agent's power.

## Overview
Skills are self-contained folders or entries containing instruction files, execution configurations, and tools. They allow the agent to execute specific workflows, integrate with external APIs, interact with local tools, run browser scraping, and invoke subagents.

## Core Capabilities & Directory Structure
Skills reside in the project-wide `.skills/` or `skills/` directory, structured as follows:
```
skills/
├── web-scraping/
│   ├── SKILL.md
│   └── scraping.json
├── code-interpreter/
│   ├── SKILL.md
│   └── run.py
└── custom-skill/
    └── SKILL.md
```

Each skill folder must contain:
1. `SKILL.md` - Main markdown file documenting frontmatter configuration, usage, API, parameters, and requirements.
2. Optional scripts, resources, or configurations.

## Memory System
Self-learning memory runs alongside agent execution:
- **Global Memory**: Persistent user-centric memory stored in the user directory (`~/.local/share/llama-manager/global_memory.json`).
- **Project Memory**: Workspace-specific memory stored in the local project workspace (`.llama-manager-memory.json`).
- Updates dynamically as tasks complete, maintaining state across restarts.

## Subagents
Agents can spawn subagents to run concurrent tasks:
- Subagents receive scoped instructions and execute tasks in the background.
- Communication with subagents uses message passing.
