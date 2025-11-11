# Claude Code Configuration

This directory contains Claude Code agents and skills following Anthropic's multi-agent architecture best practices.

## Structure

```
.claude/
├── agents/           # Research and exploration agents
│   ├── codebase-explorer.md
│   ├── solana-researcher.md
│   └── rust-pattern-analyzer.md
└── skills/           # Procedural knowledge and workflows
    ├── rust-solana-development/
    ├── tally-actions-development/
    ├── tally-sdk-development/
    ├── tally-cli-development/
    ├── htmx-template-development/
    └── solana-security-audit/
```

## Agents vs Skills

### Agents (Research/Exploration)
Agents use separate context windows to explore and research, returning condensed findings to the main conversation:

- **codebase-explorer**: Explores codebases to understand architecture, patterns, and dependencies. Returns condensed architectural summaries.
- **solana-researcher**: Researches Solana/Anchor best practices and troubleshooting via MCP. Returns implementation guidance.
- **rust-pattern-analyzer**: Analyzes code for duplication and refactoring opportunities. Returns condensed analysis with recommendations.

**When to use agents:**
- Understanding codebase structure before implementing
- Researching best practices for new features
- Finding existing implementations to avoid duplication
- Analyzing patterns for refactoring

### Skills (Procedural Knowledge)
Skills provide domain expertise that Claude discovers and loads when relevant:

- **rust-solana-development**: Solana Anchor program development expertise
- **tally-actions-development**: Axum web service and dashboard development
- **tally-sdk-development**: SDK interface layer development
- **tally-cli-development**: CLI tool development with clap
- **htmx-template-development**: HTMX/Askama template development
- **solana-security-audit**: Security audit resolution workflows

**When skills activate:**
- Claude automatically loads skills when their description matches the task
- Skills provide instructions, patterns, and workflows
- Use progressive disclosure (SKILL.md → reference files → scripts)

## Progressive Disclosure

Skills use progressive disclosure to manage context efficiently:

1. **Level 1**: Skill description (loaded at startup)
2. **Level 2**: SKILL.md content (loaded when skill activates)
3. **Level 3**: Reference files (loaded as needed)
4. **Level 4**: Scripts and detailed docs (loaded on demand)

Example:
```
rust-solana-development/
├── SKILL.md              # Core instructions (<500 lines)
└── reference/
    └── patterns.md       # Detailed patterns (on demand)
```

## Architecture Principles

Based on Anthropic's research on multi-agent systems:

1. **Separation of Concerns**: Agents research, Skills provide knowledge
2. **Conciseness**: Assume Claude is smart, be brief
3. **Progressive Disclosure**: Load information only when needed
4. **Token Efficiency**: Minimize context pollution
5. **Condensed Returns**: Agents return <2k token summaries from extensive exploration

## Best Practices

### For Agents
- Focus on exploration and research, not implementation
- Return condensed findings (500-1500 tokens)
- Use separate context for investigation
- Provide specific file references and actionable insights

### For Skills
- Keep SKILL.md under 500 lines
- Use third-person voice in descriptions
- Include specific triggers and keywords
- Break complex content into reference files
- Provide workflows with checkboxes for complex tasks

## References

- [Multi-Agent Research System](https://www.anthropic.com/research/building-effective-agents)
- [Agent Skills Overview](https://www.anthropic.com/research/agent-skills)
- [Context Engineering](https://www.anthropic.com/research/context-engineering)
