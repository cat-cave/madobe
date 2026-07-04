# m4-dependabot-contract-check Notes

- Added `scripts/dependabot-contract-check.sh` to enforce the required `.github/dependabot.yml` contract.
- The check requires `version: 2`, an `updates:` list, and weekly `github-actions` and `cargo` entries rooted at `/`.
- Wired the check through `just dependabot-contract-check` and into `just check`.
- macOS validation is not required for this repository metadata contract check.
