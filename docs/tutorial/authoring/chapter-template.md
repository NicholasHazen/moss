# Chapter template

[Authoring guide](README.md) · [Reader guide home](../README.md)

Copy the structure below into a new chapter only when the next increment has a
clear purpose. Replace every bracketed placeholder, resolve the relative links
for its destination, and remove author instructions before publishing it. Keep
existing chapters and anchors stable. The template is a drafting aid, not a
requirement to make every reader page sound identical.

---

# [Chapter number]. [Concrete change in Moss]

[Guide home](../README.md) · Previous: [chapter title] · Next: [chapter title]

**Status:** [Ready, Future, or Reviewed; name missing preparation if Future.]
**Source/base:** [Actual source and test files; revision and relevant working changes.]
**Reviewed:** [Date or not reviewed.] **Verified:** [Date and concise evidence, or not run.]

[Open with the result we want to observe or the concrete problem in the current
code. Reuse a familiar Moss example. Explain why the existing rule or data
cannot yet produce that result. Resolve this question at the end of the chapter.]

**Start here:** [One file, one edit, one useful stopping point. State preparation
the agent will finish first; do not make the reader infer missing APIs.]

## On this page

- [The change and its reason](#the-change-and-its-reason)
- [Checkpoint A: one executable increment](#checkpoint-a-one-executable-increment)
- [Optional context: one nearby idea](#optional-context-one-nearby-idea)
- [Review and stop](#review-and-stop)

## The change and its reason

[Connect the current behavior to the proposed change in a few paragraphs. Name
one main Rust/ECS concept and explain it at the point of use. Follow the same
creature, patch, tick sequence, or test world through the explanation. Link
reusable background instead of requiring a separate reading assignment.]

[Distinguish the paired rule from agent-owned installation, query plumbing,
fixtures, or inspection. Label design proposals and boundaries explicitly.]

## Checkpoint A: one executable increment

[Action: name the exact file and symbol, placement, required imports, and whether
the following code is a complete addition, replacement, test, or excerpt. Supply
the complete worked answer here or in a clearly linked section.]

[Place the correctly labeled, fenced code example here. Do not leave a
compilable-looking placeholder or invented existing API in the published guide.]

[Explain the consequential lines in connected prose. Focus on why this data,
borrow, query, operation, or ordering produces the result.]

**Run from the repository root:** [Exact tested command and test filter.]

**Expected:** [Specific test name/count and literal state or result. If this is
an intended red checkpoint, name the expected failure.]

[Why this result is evidence for the intended change, and what it cannot yet
prove. Distinguish helper, installed-schedule, and browser checks.]

**If the result differs:** [One targeted diagnostic for the likely failure, such
as an unmatched filter, missing import, or incorrect schedule position. Tell the
reader when to send the output instead of expanding the debugging task.]

**Verification record:** [Observed result, date, base, and whether this ran in
the live project or an isolated copy. Say not run for unexecuted steps.]

## Optional context: one nearby idea

[One adjacent variation that deepens the current concept without requiring a
new feature. Make its optional nature clear.]

**Check:** [Immediately available answer, literal expectation, or exact test
that checks it. Do not require the reader to earn access to the answer.]

## Review and stop

[Resolve the opening problem using the state just demonstrated. Name the
browser observation if this completes a behavior; distinguish planned
acceptance from anything already observed. End at this increment.]

**Send for review:**

> [Checkpoint name] is green. Review my rule, test expectations, and Rust usage.
> Handle the remaining plumbing and verification with me, update NOW.md, and
> keep the next behavior for a separate paired step.

[Guide home](../README.md) · [One recommended next chapter, to begin after review.]
