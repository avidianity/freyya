# evidence-bundle

**Invariant**:
Every pull request body has the sections of the Evidence bundle template: Tier, Task, Fail-first test, Checks, Safety proof, Runtime evidence, and Gaps.
Each section has content, or the words "Not applicable" with a reason.
The check verifies that the revision cited in the Evidence bundle matches the pull request's exact head commit SHA.
Forge required status checks are the authoritative proof for CI checks, never pull request prose alone.
For Tier 2 and above, Safety proofs and user-visible runtime evidence reference machine-generated CI or verifier artifacts stamped with that exact head commit SHA.
Fail-first test proof stays lightweight in Day-1 CI (pull request output, re-verified by an Independent reviewer at Tier 3) rather than forcing double-checkout CI replay.

Kit spec: [checks/evidence-bundle.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/evidence-bundle.md).

## Implemented by

- Command: `cargo xtask evidence-bundle --body-file <FILE> --head-sha <SHA> --tier <N>`.
- It requires the sections Tier, Task, Fail-first test, Checks, Safety proof, Runtime evidence, and Gaps, each with content.
- It requires a `Revision:` line with the exact head commit SHA, and no template placeholders left.
- From Tier 2, the Safety proof and the Runtime evidence each hold a `https://github.com/<OWNER>/<REPO>/actions/runs/<ID>` link and the first seven characters of the head SHA; the Runtime evidence may instead start with "Not applicable:" and a reason; Tier 2 may say "unproven" in the Safety proof and list it under Gaps.
- Template: `.github/pull_request_template.md`, from [docs/templates/pr-evidence.md](../templates/pr-evidence.md).

Runs in: CI, in `.github/workflows/policy.yml`, which reads the pull request body as data.
