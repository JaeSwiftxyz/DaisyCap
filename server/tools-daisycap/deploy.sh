#!/usr/bin/env bash
# Publishes the DaisyCap page (tools/daisycap/ in this repo) at https://jaeswift.xyz/tools/daisycap/.
# Safe to re-run.
#
#   server/tools-daisycap/deploy.sh root@81.0.219.227
#
# The nginx site file is backed up first. nginx reloads only when `nginx -t` passes. If the test
# fails, the backup goes back and nothing reloads.
set -euo pipefail

host="${1:?usage: deploy.sh user@host}"
here="$(cd "$(dirname "$0")" && pwd)"
page="$here/../../tools/daisycap"
site=/etc/nginx/sites-available/jaeswift.xyz
stamp="$(date +%Y%m%d-%H%M%S)"

ssh "$host" 'install -d -m 755 /var/www/daisycap-tools'
tar -C "$page" -cf - . | ssh "$host" 'tar -C /var/www/daisycap-tools -xf -'
scp -q "$here/daisycap-tools.conf" "$host:/etc/nginx/snippets/daisycap-tools.conf"

ssh "$host" bash -s -- "$site" "$stamp" <<'REMOTE'
set -euo pipefail
site="$1"; stamp="$2"
cp -a "$site" "/root/$(basename "$site").bak-$stamp"
if ! grep -q 'snippets/daisycap-tools.conf' "$site"; then
    sed -i '0,/^    include \/etc\/nginx\/snippets\/daisycap-downloads.conf;/s//&\n    include \/etc\/nginx\/snippets\/daisycap-tools.conf;/' "$site"
fi
if ! nginx -t 2>/tmp/nginx-test.log; then
    cp -a "/root/$(basename "$site").bak-$stamp" "$site"
    echo "nginx test failed; restored $site" >&2
    cat /tmp/nginx-test.log >&2
    exit 1
fi
systemctl reload nginx
echo "published the page; site backup at /root/$(basename "$site").bak-$stamp"
REMOTE
