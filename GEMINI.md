# Rubedo Workspace Rules & Mandatory Quality Gates

See [AGENTS.md](file:///c:/dev/Rubedo/AGENTS.md) for full guidelines.

## Mandatory Post-Code Task Audit Protocol
Whenever you finish implementing any code task, feature, crate, or phase deliverable:
You MUST execute the master post-task audit pipeline before concluding:

```powershell
powershell -ExecutionPolicy Bypass -File .agents/scripts/run_post_task_audits.ps1
```

This ensures `critical-code-reviewer` and `security-auditor` checks are automatically run.
