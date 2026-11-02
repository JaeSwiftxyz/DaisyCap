#!/usr/bin/env bash
# Publishes a DaisyCap DMG at jaeswift.xyz/daisycap/releases/latest/download/DaisyCap.dmg, and keeps
# each version's copy under /daisycap/downloads/. Safe to re-run.
#
#   server/downloads/deploy.sh root@81.0.219.227 dist/DaisyCap.dmg
#
# The nginx site file is backed up first. nginx reloads only when `nginx -t` passes. If the test
# fails, the backup goes back and nothing reloads.
set -euo pipefail

host="${1:?usage: deploy.sh user@host path/to/DaisyCap.dmg}"
dmg="${2:?usage: deploy.sh user@host path/to/DaisyCap.dmg}"
here="$(cd "$(dirname "$0")" && pwd)"
version="$(sed -n 's/^version = "\(.*\)"/\1/p' "$here/../../Cargo.toml" | head -1)"
site=/etc/nginx/sites-available/jaeswift.xyz
stamp="$(date +%Y%m%d-%H%M%S)"

ssh "$host" 'install -d -m 755 /var/lib/daisycap-downloads'
scp -q "$dmg" "$host:/var/lib/daisycap-downloads/DaisyCap-$version.dmg"
scp -q "$here/daisycap-downloads.conf" "$host:/etc/nginx/snippets/daisycap-downloads.conf"

ssh "$host" bash -s -- "$version" "$site" "$stamp" <<'REMOTE'
set -euo pipefail
version="$1"; site="$2"; stamp="$3"
dir=/var/lib/daisycap-downloads
cp -f "$dir/DaisyCap-$version.dmg" "$dir/DaisyCap.dmg.new"
mv -f "$dir/DaisyCap.dmg.new" "$dir/DaisyCap.dmg"
chmod 644 "$dir"/*.dmg
cp -a "$site" "/root/$(basename "$site").bak-$stamp"
if ! grep -q 'snippets/daisycap-downloads.conf' "$site"; then
    sed -i '0,/^    include \/etc\/nginx\/snippets\/daisycap-share.conf;/s//&\n    include \/etc\/nginx\/snippets\/daisycap-downloads.conf;/' "$site"
fi
if ! nginx -t 2>/tmp/nginx-test.log; then
    cp -a "/root/$(basename "$site").bak-$stamp" "$site"
    echo "nginx test failed; restored $site" >&2
    cat /tmp/nginx-test.log >&2
    exit 1
fi
systemctl reload nginx
echo "published DaisyCap-$version.dmg; site backup at /root/$(basename "$site").bak-$stamp"
REMOTE
