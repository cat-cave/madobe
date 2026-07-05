# m4-nightly-full-workflow-dispatch Credential Blocker

The hosted nightly workflow could not be dispatched from `main`.

Command:

```sh
gh workflow run nightly.yml --repo cat-cave/madobe --ref main
```

Observed:

```text
could not create workflow dispatch event: HTTP 403: Resource not accessible by integration (https://api.github.com/repos/cat-cave/madobe/actions/workflows/306353539/dispatches)
```

Platform: Linux/Nix worktree at `/home/trevor/projects/madobe/.qd/worktrees/m4-nightly-full-workflow-dispatch`.

Missing condition: the active `gh` credential is `trevor-workstation[bot]` and lacks GitHub Actions workflow dispatch permission for `cat-cave/madobe`.

Evidence:

- `evidence/m4-nightly-full-workflow-dispatch/commands.log`
- `evidence/m4-nightly-full-workflow-dispatch/nightly-run.md`

Unblock path:

1. Authenticate `gh` with a credential that can create workflow dispatch events for `cat-cave/madobe`.
2. Rerun `gh workflow run nightly.yml --repo cat-cave/madobe --ref main`.
3. Identify and watch the created run to completion.
4. Update `evidence/m4-nightly-full-workflow-dispatch/nightly-run.md` with the run id, URL, commit SHA, conclusion, job summaries, and any failure logs.
