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

- Command: `cargo xtask evidence-bundle --body-file <FILE> --head-sha <SHA> --tier <N> --repo <OWNER/NAME> [--ci-run-url <URL>]`.
- It requires the sections Tier, Task, Fail-first test, Checks, Safety proof, Runtime evidence, and Gaps, each with content.
- It reads only the visible text: HTML comments (`<!-- ... -->`) count as empty.
- A `Revision:` line is optional; when present it must be the exact head commit SHA. The policy workflow supplies the head SHA itself, so the body does not have to repeat it after each push (Owner decision, 2026-11-28, to remove manual steps). It requires no template placeholders or template lines (such as an untouched `<Link to the task ...>` line) left.
- A run URL must point to this repository (`--repo`, from the workflow).
- The policy workflow asks GitHub for the `ci` run of the head commit and passes its URL as `--ci-run-url`; that run proves the Safety proof for Tier 2 and above, so the body needs no link. A link in the body still works.
- Known limits: a run link written in the body is not checked against GitHub, so a made-up run number in the body passes; words like "TODO" count as content; the first `Revision:` line wins. The Independent reviewer and the Owner read the bundle too.
- From Tier 2, the Safety proof and the Runtime evidence each hold a `https://github.com/<OWNER>/<REPO>/actions/runs/<ID>` link and the first seven characters of the head SHA; the Runtime evidence may instead start with "Not applicable:" and a reason; Tier 2 may say "unproven" in the Safety proof only when the Gaps section lists it.
- Template: `.github/pull_request_template.md`, from [docs/templates/pr-evidence.md](../templates/pr-evidence.md).

Runs in: CI, in `.github/workflows/policy.yml`, which reads the pull request body as data.
