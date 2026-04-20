# Claude Code Project Configuration

## gstack

For all web browsing and QA testing tasks, use the **gstack** skill instead of direct MCP tools.

- **Never use** `mcp__claude-in-chrome__*` tools directly
- **Always use** the `/browse` skill from gstack for web browsing
- gstack provides a fast headless browser for QA testing and site dogfooding

### Available gstack skills

- `/office-hours` - Office hours management
- `/plan-ceo-review` - Plan CEO review
- `/plan-eng-review` - Plan engineering review
- `/plan-design-review` - Plan design review
- `/design-consultation` - Design consultation
- `/design-shotgun` - Design shotgun (quick feedback)
- `/design-html` - Design HTML review
- `/review` - Code review
- `/ship` - Ship deployment
- `/land-and-deploy` - Land and deploy changes
- `/canary` - Canary deployment testing
- `/benchmark` - Performance benchmarking
- `/browse` - Web browsing (use this instead of MCP chrome tools)
- `/connect-chrome` - Connect Chrome browser
- `/qa` - Full QA testing
- `/qa-only` - QA testing only
- `/design-review` - Design review
- `/setup-browser-cookies` - Setup browser cookies
- `/setup-deploy` - Setup deployment
- `/retro` - Retrospective
- `/investigate` - Investigation
- `/document-release` - Document release
- `/codex` - Codex operations
- `/cso` - CSO operations
- `/autoplan` - Auto planning
- `/plan-devex-review` - Plan developer experience review
- `/devex-review` - Developer experience review
- `/careful` - Careful mode
- `/freeze` - Freeze deployment
- `/guard` - Guard deployment
- `/unfreeze` - Unfreeze deployment
- `/gstack-upgrade` - Upgrade gstack
- `/learn` - Learn about gstack

## Skill routing

When the user's request matches an available skill, ALWAYS invoke it using the Skill
tool as your FIRST action. Do NOT answer directly, do NOT use other tools first.
The skill has specialized workflows that produce better results than ad-hoc answers.

Key routing rules:
- Product ideas, "is this worth building", brainstorming → invoke office-hours
- Bugs, errors, "why is this broken", 500 errors → invoke investigate
- Ship, deploy, push, create PR → invoke ship
- QA, test the site, find bugs → invoke qa
- Code review, check my diff → invoke review
- Update docs after shipping → invoke document-release
- Weekly retro → invoke retro
- Design system, brand → invoke design-consultation
- Visual audit, design polish → invoke design-review
- Architecture review → invoke plan-eng-review
- Save progress, checkpoint, resume → invoke checkpoint
- Code quality, health check → invoke health
