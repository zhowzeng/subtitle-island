# Google Technical Writing Reference

This reference condenses the Google Technical Writing courses into operational rules for AI-assisted technical writing. It is a guide, not a substitute for the official material.

Official sources:

- Technical Writing One: https://developers.google.com/tech-writing/one
- Technical Writing Two: https://developers.google.com/tech-writing/two
- Tech Writing for Accessibility: https://developers.google.com/tech-writing/accessibility
- Writing Helpful Error Messages: https://developers.google.com/tech-writing/error-messages
- Google Developer Documentation Style Guide: https://developers.google.com/style

## Technical Writing One

Technical Writing One focuses on clarity, concision, terminology, active voice, sentence structure, lists, paragraphs, audience, document scope, punctuation, and Markdown.

Use these rules:

- Choose one term for each concept and use it consistently.
- Define abbreviations and acronyms before using them repeatedly.
- Replace ambiguous pronouns with specific nouns when the referent is unclear.
- Prefer active voice when it identifies the actor and shortens the sentence.
- Use passive voice only when the actor is irrelevant, unknown, or less important than the object.
- Reduce long sentences by splitting independent ideas, converting embedded lists into bullets, and deleting needless words.
- Put strong verbs close to their subjects.
- Turn sequential tasks into numbered lists.
- Turn unordered sets into bulleted lists.
- Introduce every list with enough context.
- Keep list items parallel.
- Focus each paragraph on one topic.
- Start paragraphs with a lead sentence that frames the point.
- State key points early.
- Identify the target audience and what they already know.
- Account for the curse of knowledge: writers often omit context that beginners need.
- Replace idioms when the audience might include non-native speakers.
- State document scope, audience, and non-goals when misinterpretation is likely.
- Break long topics into sections with informative headings.
- Use Markdown structure deliberately; do not use formatting as decoration.

## Technical Writing Two

Technical Writing Two focuses on drafting, self-editing, organizing larger docs, illustrations, sample code, documentation types, tutorials, and LLM-assisted writing.

Use these rules:

- Treat writing as iterative: draft first, then edit for structure, clarity, and precision.
- Adopt the local style guide or a public style guide for consistency.
- Read from the audience's perspective.
- Use personas sparingly to expose assumptions, not to narrow the doc too much.
- Read text aloud or change review context to find awkward phrasing.
- Ask for peer review when accuracy or usability matters.
- Create an outline before writing large docs.
- Use the outline to group related concepts and tasks.
- Introduce concepts when the reader needs them, not when the author remembers them.
- Explain why a task matters before asking readers to perform it.
- Alternate concept and application when teaching complex material.
- For large docs, provide navigation through introductions, summaries, headings, table of contents, related links, and next steps.
- Prefer task-based headings for beginner and mixed audiences.
- Use progressive disclosure: give essential information first; link or defer advanced details.
- Introduce scope, prerequisites, and exclusions near the beginning.
- Make figures carry useful information, not decoration.
- Write figure captions that explain the point of the figure.
- Use visual emphasis to direct attention to the relevant part of a diagram.
- Include big-picture diagrams when readers need system context.
- Keep sample code focused on the teaching goal.
- Show enough code for correctness, but remove unrelated complexity.
- Use LLMs for drafting, editing, formatting, and summarizing, but verify accuracy and style.

## Accessibility

Accessibility guidance focuses on inclusive design, alt text, color contrast, accessible visuals, inclusive language, and accessibility-focused editing.

Use these rules:

- Assume readers use different devices, input methods, assistive technology, and reading contexts.
- Write meaningful link text; avoid "click here" and bare URLs when a descriptive label is better.
- Provide alt text for informative images. Describe the information the reader needs, not every visible detail.
- Mark decorative images as decorative when the platform supports it.
- Do not rely on color alone to convey state, warnings, or categories.
- Check color contrast when editing visual docs or UI-facing content.
- Keep diagrams simple enough to understand with labels and text alternatives.
- Use inclusive, respectful language.
- Make headings, lists, and tables structurally correct so screen readers can navigate them.

## Error Messages

Helpful error messages are technical writing under pressure. They must be concise, accurate, and actionable.

Use these rules:

- Answer "what went wrong?" and "how does the user fix it?"
- Identify the cause when known.
- Identify invalid user input when safe and useful.
- State requirements and constraints directly.
- Provide a concrete recovery action.
- Include examples for required formats when helpful.
- Use terminology the target audience understands.
- Avoid double negatives.
- Avoid blame. Focus on the state and the fix.
- Avoid vague errors such as "Something went wrong" when a more specific cause is available.
- Avoid exposing secrets, private paths, or sensitive internals.
- Keep the tone calm and direct.

## Editing Pass Order

Use this order for substantial edits:

1. Audience and goal: identify the reader, task, and success criteria.
2. Scope: remove unrelated material; add missing prerequisites or non-goals.
3. Structure: reorder sections around the reader's path.
4. Headings: make headings task-based and specific.
5. Paragraphs: keep one topic per paragraph with a clear lead sentence.
6. Sentences: prefer active voice, concrete subjects, and direct verbs.
7. Words: replace vague or inconsistent terms.
8. Lists and tables: choose the correct format and enforce parallel structure.
9. Examples and code: remove unrelated complexity and add verification where useful.
10. Accessibility: check links, images, color-dependent meaning, and structure.
11. Final polish: remove filler and check tone.

## Common Rewrites

Prefer:

- "To start the app, run `bun run tauri dev`."
- "If the command fails because dependencies are missing, run `bun install`."
- "This guide assumes you know basic Rust and Svelte."
- "This document does not cover packaging or auto-update."

Avoid:

- "It should be noted that you can start the application by executing the following command."
- "Run `bun run tauri dev` if you want to start the app." when the condition should come first.
- "We will now explain..." when the section can state the point directly.
- "The file is read by the system." when the actor matters.

## Repository Documentation Bias

For repository docs, optimize for future maintainers:

- Prefer durable rules over narrative history.
- Link to source files or commands when they are stable.
- State verification commands.
- Name assumptions explicitly.
- Keep speculative future work out of normative docs unless it is labeled as non-goal or future work.
- Avoid restating code line by line.
