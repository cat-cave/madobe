# qd Merge Summary SHA Contract Notes

The qd report checker already requires `reports/qd/*/completion.json`
`commits[]` entries to be full, reachable commit SHAs. This node applies the
same durability expectation to qd merge-run summaries in the portable roadmap
export.

Nine historical merge summaries used short SHAs. They now record the full
squash commit SHA:

- `6e1c6c0` -> `6e1c6c09f8330e02f5c022e093aff19934b089b3`
- `266e6c0` -> `266e6c06d8f0096d05465c5f2021b394e6a1be35`
- `a558cc1` -> `a558cc19e1ab0c57707909fefa12d6a33c240598`
- `ff33895` -> `ff33895fbe10acde6e51a2c9529d67b436228798`
- `73236c1` -> `73236c16391d6bcab27b5f88729406551ca3ed9d`
- `cbba43a` -> `cbba43ad71695d0d7ced8229a98abce636dbf859`
- `6dfaab5` -> `6dfaab506575fae6a40f363415a09bdbea502383`
- `ad45b67d` -> `ad45b67933eda62183a87f9fe513c68d580a4b36`

The `6e1c6c0` prefix appeared in two M0 merge summaries. The final
`ad45b67d` value was resolved from PR #110 and its existing completion report.

The checker rejects any `runs[].summary` beginning with the merge-recorded
squash commit prefix unless the commit token is exactly a 40-character
lowercase hexadecimal SHA.
