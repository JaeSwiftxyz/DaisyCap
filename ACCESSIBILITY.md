# Accessibility

DaisyCap is a screen recorder for Mac. A recording is no use if the app is hard to drive or the result is hard to watch, so accessibility runs through both. This covers the app, the download page in `site/` and the docs. It is for Mac users with low vision, motor difficulties, cognitive load, deafness or hearing loss, and anyone using a keyboard, VoiceOver or reduced motion. This is a working statement, not a conformance claim.

## Priorities

- A keyboard path through recording. Starting, stopping, pausing and cancelling have global shortcuts you can rebind. The menu bar repeats the common actions.
- Captions. The editor can transcribe on device with Whisper or Apple Speech and show captions in the export. Captions can be edited and saved as subtitles.
- Motion that can settle. The splash follows Reduce Motion, and the site follows `prefers-reduced-motion`.
- Readable contrast and layouts that hold at 390 px, with visible focus on the site.
- Plain language in the app, the site and the docs.

WCAG 2.2 AA is the target for the download page. Nobody has done a formal audit, so treat it as the bar I aim at, not a verified claim. The Mac app is judged against macOS conventions.

## Contributor expectations

For user-facing changes:

- Check the keyboard path. New actions need a key or a menu item, and focus must not disappear.
- Check the page at 390 px and at desktop width, in light and dark.
- Keep focus visible and use `prefers-reduced-motion` for anything that moves.
- Caption new demo videos or recordings that contain speech.
- Give images and icons useful alt text, or `aria-hidden` when the text already says it.
- Do not use colour alone to carry meaning. Pair it with a label, shape or pattern.
- Run `npm run check` and `npm run build` for `site/` changes.
- In the PR, say what you checked by keyboard, what you checked with VoiceOver, and what you could not check. Screenshots help.

## Reporting accessibility issues

Email jae@jaeswift.xyz with "DaisyCap accessibility" in the subject. Opening an issue also works, but email gets a faster reply. You do not need to name a disability or use standards codes.

Useful details:

- what you were trying to do
- the page or app screen, with a URL or project file if there is one
- what happened and what you expected
- macOS version, browser and version if it is the site
- the assistive technology and settings, if any
- a screenshot or recording if you have one, optional

### Severity

- Blocker: the task cannot be finished. For example, the download button cannot be reached by keyboard, or a recording cannot be started without a pointer.
- Major: the task is possible but slow, confusing or error-prone. For example, VoiceOver reads a control as "button" with no name, or focus jumps somewhere unexpected.
- Minor: a rough edge with a workaround. For example, cramped text at 390 px or weak contrast on secondary text.

I confirm or change the severity during triage. You do not have to pick one.

### How we respond

- I acknowledge within 3 working days.
- I confirm the severity and next step within 5 working days, and give a workaround if one exists.
- I send an update at least every 2 weeks while the issue is open.
- A fix gets a note in the release notes. You can test a build before it ships if you want to.
- If I decline a report, I say why.

## Supported environments

- macOS 14 and later, Apple Silicon and Intel
- Safari on macOS, which is what the download page is tested in. Current Chrome and Firefox should work but are not routinely tested.
- Phone widths down to 390 px
- Keyboard and pointer input
- VoiceOver: partial, with gaps listed below
- Reduce Motion: respected by the splash and the site

Not yet tested: Switch Control, Voice Control, Full Keyboard Access across every screen, high contrast display modes, and browser zoom above 200 percent. Treat support there as unknown rather than promised.

## Known limitations

- VoiceOver support is incomplete. The editor draws its own controls with egui, so many expose a generic role and some have no label. Menus and global shortcuts cover the main recording actions. If a control is unusable, say which one and what you needed it for.
- Dragging clips, trim handles and mask bounds needs a pointer. Split, zoom, mask, callout, undo, delete and seeking all have keys, and Space plays or pauses.
- Drag-selecting a screen area needs a pointer. Recording a display or window does not.
- Captions depend on the source audio. Whisper can mishear names and technical words, so captions need a read before export.
- The demo on the download page is animated HTML, not video. It has a text description and labelled controls, but it is one fixed animation rather than a set of steps you can drive.
- No formal audit has run. The list above is what testing has found, not proof that everything else works.

## Feedback and improvements

Tell me if this statement or the way the project handles accessibility could be better. Email jae@jaeswift.xyz with "DaisyCap accessibility" in the subject, or open an issue. Active barriers belong in the reporting process above. Suggestions about wording, scope or process are welcome in the same place.
