---
name: tech-writing
description: Revise, write, or review technical documentation using Google Technical Writing course principles. Use when an assistant or writer is asked to improve docs, README content, architecture notes, tutorials, how-to guides, API or CLI docs, developer-facing explanations, error messages, accessibility-oriented docs, sample-code explanations, or any text that should be clearer, more precise, better structured, more audience-aware, or easier to scan.
---

# Tech Writing

Use this skill to produce technical writing that is precise, reader-centered, and easy to act on.

Base the work on Google Technical Writing guidance, but adapt it to the repository's style and the user's language preference. Prefer concise edits over broad rewrites unless the document structure is the problem.

For the detailed source-derived checklist, read `references/google-tech-writing.md` when the task involves a substantial rewrite, a new doc, an error message, sample code, accessibility, or a review with findings.

## Workflow

1. Identify the reader, goal, and expected prior knowledge.
2. Define the document scope:
   - What the document covers.
   - What the reader should already know.
   - What the document intentionally does not cover.
3. Choose the document shape:
   - Tutorial: ordered learning path with prerequisites and verification.
   - How-to: task-focused steps for a known goal.
   - Concept: explanation of a system, model, or tradeoff.
   - Reference: complete facts, options, commands, fields, or APIs.
   - Error message: problem plus fix.
4. Draft or revise from the reader's task outward.
5. Edit for clarity, structure, brevity, and consistency.
6. Verify that headings, lists, examples, and code samples support the reader's next action.

## Core Rules

- Use consistent terminology. Define unfamiliar terms before relying on them.
- Prefer active voice when it clarifies the actor.
- Put conditions before instructions.
- Use second person for instructions when appropriate: "you".
- Keep each sentence focused on one idea.
- Keep each paragraph focused on one topic.
- Start paragraphs with a useful lead sentence.
- Put the key point near the start of each section.
- Use numbered lists for ordered steps.
- Use bulleted lists for unordered items.
- Use tables only when readers must compare structured attributes.
- Format code, commands, filenames, flags, fields, and literals in code font.
- Remove filler, throat-clearing, vague intensifiers, and unsupported claims.
- Preserve necessary nuance; do not shorten text by deleting constraints the reader needs.

## Document Structure

For new or heavily revised docs, include only the sections the reader needs.

Prefer this order for task docs:

1. Goal or outcome.
2. Audience and prerequisites, when not obvious.
3. Scope and non-goals, when misinterpretation is likely.
4. Procedure or decision rules.
5. Examples.
6. Verification or success criteria.
7. Troubleshooting or next steps.

Prefer task-based headings. Headings should tell readers what they can do or learn, not merely name an internal component.

Avoid placing one heading immediately after another. Add a brief orienting sentence when a section needs context.

## Code Samples

Use sample code to teach one point at a time.

- Make samples minimal but complete enough to run or adapt.
- Show realistic names and values.
- Explain why the sample matters before or after the code block.
- Remove unrelated cleverness.
- Include expected output when it helps verification.
- Keep comments sparse and useful.

## Error Messages

A helpful error message answers two questions:

1. What went wrong?
2. How does the user fix it?

Make error messages specific, actionable, and aimed at the target audience. Include the invalid input, violated requirement, expected format, or next command when that information is available. Avoid blame, jokes, double negatives, and vague messages such as "failed" without cause or recovery.

## Accessibility

Make docs usable with assistive technology and by readers with different contexts.

- Use meaningful link text.
- Add useful alt text for informative images.
- Do not rely on color alone to communicate meaning.
- Keep diagrams focused and label the relevant parts.
- Use inclusive language.
- Make headings and lists convey structure.

## Review Checklist

When reviewing a document, lead with the highest-impact issues:

- Ambiguous audience or missing prerequisites.
- Missing goal, scope, or non-goals.
- Unclear actor or passive construction that hides responsibility.
- Long sentences or paragraphs that combine multiple ideas.
- Terms used inconsistently or before definition.
- Ordered steps written as bullets, or unordered options written as steps.
- Headings that do not help navigation.
- Examples that are unrealistic, incomplete, or unrelated to the surrounding explanation.
- Error messages that omit cause or recovery.
- Accessibility issues in links, images, contrast, or structure.

## Output Style

When editing existing text, prefer one of these formats:

- Direct replacement text for small edits.
- A patch or file edit when working in a repository.
- Findings plus suggested rewrites for review-only tasks.

Do not over-explain basic grammar. Explain only the changes that affect reader comprehension, correctness, or maintainability.
