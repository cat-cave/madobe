# m4-qd-realworld-blocked-status-contract Notes

This node updates qd report validation only. It does not change product runtime behavior or claim product real-world validation.

The positive evidence is `nix develop -c just qd-reports-check` passing after an existing required-real-world report was changed to `realWorldValidation.status: "blocked"`.

The negative evidence uses a temporary copy under `/tmp` with `realWorldValidation.status: "unexpected_status"` and directly invokes the shared `validate_real_world_status` predicate. The first two negative probe attempts in `commands.log` failed due shell quoting/setup errors before the intended validator check; the final probe records the actual rejection.
