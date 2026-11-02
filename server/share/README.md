# DaisyCap share server

The far end of share links: `jaeswift.xyz/daisycap/<handle>/<video>`.

- `handle.php` claims a handle and returns its token. Only the first claim gets one.
- `upload.php` takes a video (MP4, MOV or GIF) or a JPEG poster as the raw request body.
- `delete.php` takes a video down. It needs the handle's token.
- `view.php` is the page for a video. It's unlisted (`noindex`) and carries Open Graph tags.
- `daisycap-share.conf` routes those paths in nginx. Video files are served straight from the store.

Limits: 1 GB a video, 5 GB a handle, 8 MB a poster. The server keeps 10 GB free.

Layout on the server:

- `/opt/daisycap-share/`: these scripts, and `.user.ini`, which raises the upload limits for this folder only.
- `/var/lib/daisycap-share/handles/<handle>.json`: the token's hash and the bytes the handle uses.
- `/var/lib/daisycap-share/files/<handle>/<video>/`: the video, its poster and `meta.json`.

Tokens are stored as hashes. A lost token means a lost handle; there's no recovery route yet.

Deploy (safe to re-run; the nginx site file is backed up and only reloaded when `nginx -t` passes):

    server/share/deploy.sh root@<host>

Live test (real uploads, then the video is removed; the test handle's record stays until you delete it by hand):

    cargo test --test share -- --ignored
