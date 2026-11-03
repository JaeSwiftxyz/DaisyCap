# Security policy

DaisyCap is a local-first Mac app. Recordings stay on the Mac unless you share one, so the main network features are the update feed and shared video links. If you find a hole in any of them, tell me privately so it can be fixed before the details are public.

## Supported versions

Only the latest 0.2.x release gets security fixes. The app updates itself from the signed feed, so moving to it takes one click. Older releases are unsupported.

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | :white_check_mark: |
| < 0.2   | :x:                |

## Reporting a vulnerability

Email **jae@jaeswift.xyz** with "DaisyCap security" in the subject. Send it there rather than opening a public issue, so there is time to fix it first. If email is a problem, open an issue with a title starting "Security:" and no details, and I will get in touch.

Include:

- the DaisyCap version, from DaisyCap › About DaisyCap
- your macOS version and Mac model
- what you did, what happened and what you expected
- a proof of concept, crash log or recording if you have one
- how you want to be credited, or say if you would rather stay anonymous

What happens next:

- I acknowledge the report within 3 working days.
- I decide whether to accept it within 10 working days, and say why if I decline.
- For an accepted report, I send an update at least every 7 days until it ships.
- The fix goes into the next patch release, with credit in the notes unless you ask otherwise.
- I agree a disclosure date with you. If the bug is already public, say so and I will move faster.

Good-faith testing is welcome. Stay off other people's shared videos and anything that costs money. A near miss still gets a reply.

## Scope

In scope: the DaisyCap app in this repo, the download page in `site/`, the update feed and the share service in `server/`.

Out of scope: macOS and Apple frameworks, and bugs in third-party crates. Report those upstream, though a note helps if DaisyCap is affected.

## Safe harbour

Follow this policy and I won't chase you for testing. Keep to your own recordings and data where you can. If you need to touch a shared video, name the link in your report.
