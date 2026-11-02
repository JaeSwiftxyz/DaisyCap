#!/usr/bin/env bash
# Deploys the DaisyCap share-link server to jaeswift.xyz. Safe to re-run.
#
#   server/share/deploy.sh root@81.0.219.227
#
# The nginx site file is backed up first, and nginx is only reloaded when
# `nginx -t` passes. If the test fails, the backup is put back.
set -euo pipefail

host="${1:?usage: deploy.sh user@host}"
here="$(cd "$(dirname "$0")" && pwd)"
site=/etc/nginx/sites-available/jaeswift.xyz
stamp="$(date +%Y%m%d-%H%M%S)"

ssh "$host" bash -s <<'REMOTE'
set -euo pipefail
mkdir -p /opt/daisycap-share /var/lib/daisycap-share/{handles,files,tmp}
chown -R www-data:www-data /var/lib/daisycap-share
chmod 750 /var/lib/daisycap-share /var/lib/daisycap-share/handles /var/lib/daisycap-share/tmp
chmod 755 /var/lib/daisycap-share/files
REMOTE

scp -q "$here"/lib.php "$here"/handle.php "$here"/upload.php "$here"/delete.php "$here"/view.php "$here"/cleanup.php "$host:/opt/daisycap-share/"
scp -q "$here/user.ini" "$host:/opt/daisycap-share/.user.ini"
scp -q "$here/daisycap-share.conf" "$host:/etc/nginx/snippets/daisycap-share.conf"
scp -q "$here/daisycap-share-limit.conf" "$host:/etc/nginx/conf.d/daisycap-share-limit.conf"
ssh "$host" 'printf "%s\n" "*/5 * * * * root /usr/bin/php /opt/daisycap-share/cleanup.php >/dev/null 2>&1" > /etc/cron.d/daisycap-share && chmod 644 /etc/cron.d/daisycap-share'

ssh "$host" bash -s -- "$site" "$stamp" <<'REMOTE'
set -euo pipefail
site="$1"; stamp="$2"
cp -a "$site" "/root/$(basename "$site").bak-$stamp"
if ! grep -q 'snippets/daisycap-share.conf' "$site"; then
    # Include the share routes at the top of the TLS server block (the first server_name line).
    sed -i '0,/^    server_name jaeswift.xyz www.jaeswift.xyz;/s//&\n    include \/etc\/nginx\/snippets\/daisycap-share.conf;/' "$site"
fi
if ! nginx -t 2>/tmp/nginx-test.log; then
    cp -a "/root/$(basename "$site").bak-$stamp" "$site"
    echo "nginx test failed; restored $site" >&2
    cat /tmp/nginx-test.log >&2
    exit 1
fi
systemctl reload nginx
# PHP picks up the folder's .user.ini limits after a reload.
systemctl reload php8.3-fpm
echo "deployed; backup at /root/$(basename "$site").bak-$stamp"
REMOTE
