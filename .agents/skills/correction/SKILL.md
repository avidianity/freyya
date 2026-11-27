---
name: correction
description: Turn a repeated agent mistake into a check at the lowest layer that can stop it. Use when the same mistake happens a second time, or when the Owner asks for a Correction.
argument-hint: The mistake, with links to where it happened
---

# correction

Read `docs/agents/corrections.md` before you start.

1. **Search.**
   Look for an open issue labelled `correction` for the same failure class.
   When one exists, add this occurrence to it and stop, unless the Owner asked you to build its check now.
2. **Confirm the repeat.**
   Find at least two occurrences.
   With only one, fix that one in its own change and stop.
3. **Find the cause.**
   Name the missing invariant, the confusing API, the unclear requirement, or the missing tool.
4. **Pick the layer.**
   Walk the decision order in `docs/agents/corrections.md`, and take the first layer that can stop the mistake.
   Write down why each lower layer cannot.
5. **Write the cases.**
   One input that the check must reject, and one that it must accept.
6. **Open the issue.**
   Use the Correction template in `docs/templates/correction.md`, label it `correction`, and add the owner label.
7. **Build the check** when the Owner asks for it, or when the Project's rules let agents do it alone.
   Follow the steps in the [Kit check spec README](https://github.com/avidianity/factory/blob/v0.9.0/checks/README.md): build it, prove that the failing case fails, prove that the passing case passes, and write its page in `docs/checks/`.
8. **Shrink the prose.**
   When the check merges, cut any prose rule about the same mistake to one line that links the check.

Done when the issue exists with both cases, or when its check has merged and the issue is closed.
