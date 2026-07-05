# Nightly Workflow Dispatch Evidence

Node: `m4-nightly-full-workflow-dispatch`

Workflow: `nightly.yml`
Workflow URL: <https://github.com/cat-cave/madobe/actions/workflows/nightly.yml>
Repository: `cat-cave/madobe`
Ref requested: `main`
Commit SHA resolved before dispatch: `32ef3b6798991be6f719e460812677e349644c24`

## Dispatch

Command:

```sh
gh workflow run nightly.yml --repo cat-cave/madobe --ref main
```

Result: failed before a workflow run was created.

GitHub response:

```text
could not create workflow dispatch event: HTTP 403: Resource not accessible by integration (https://api.github.com/repos/cat-cave/madobe/actions/workflows/306353539/dispatches)
```

## Run

Run id: none
Run URL: none
Conclusion: no run created
Commit SHA: `32ef3b6798991be6f719e460812677e349644c24`

`gh run list --repo cat-cave/madobe --workflow nightly.yml --branch main --event workflow_dispatch --limit 10 --json databaseId,displayTitle,headSha,createdAt,status,conclusion,url,workflowName,event,headBranch` returned `[]` immediately after the failed dispatch, so there was no run to watch.

## Job Summaries

No job summaries are available because GitHub did not create a run.

## Failure Notes

The active `gh` account is `trevor-workstation[bot]`. `gh auth status` showed an active GitHub token, but the dispatch API returned `HTTP 403: Resource not accessible by integration`.

Recommended unblock path: rerun the dispatch with a GitHub credential that has permission to create Actions workflow dispatch events for `cat-cave/madobe`, then record the resulting run id, URL, commit SHA, conclusion, and job summaries here.
