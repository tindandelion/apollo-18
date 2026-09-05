---
name: feature-done
description: Run the completion gates, squash-merge the current branch into main, and delete the local feature branch.
disable-model-invocation: true
---

# Feature done

Finish the current feature locally. The branch present when this skill starts is the **feature branch**.

1. Run these preflight checks:
   - The feature branch is not `main` and `main` exists locally.
   - The working tree has no tracked, staged, or untracked changes.
   - `main` is an ancestor of the feature branch. If it is not, stop and ask the user to update the feature branch from `main`; the completion gates must run against the resulting integration.
   - Record the feature branch name and commit. Stop if either changes before the merge.

2. Read the repository instructions and the applicable spec and ticket. Enumerate every repository quality-gate command and every ticket-specific check required for completion. Run all of them from the documented working directories. A skipped, unavailable, inconclusive, or failed check blocks completion: stop on the first blocker and report it without changing branches.

3. Confirm the working tree is still clean and the recorded feature branch and commit are unchanged.

4. Propose the exact squash commit message and wait for the user's confirmation, as required by the repository commit policy.

5. After confirmation, repeat the preflight checks, then:
   - Switch to `main`.
   - Run `git merge --squash <feature-branch>`.
   - Confirm the staged result is non-empty and matches the feature branch tree.
   - Commit with the confirmed message.
   - Confirm the commit succeeded and `main` is clean.

6. Delete the local feature branch with `git branch -D <feature-branch>`. Squash merges require forced local deletion because Git does not record the feature branch as an ancestor.

If a merge or commit step fails, preserve the recoverable Git state, report the failure and current status, and keep the feature branch. Leave pushing `main` and deleting any remote branch to the user.
