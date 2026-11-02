<?php
// Deletes each shared video an hour after upload, and gives its bytes back to its handle.
// deploy.sh installs this to run every five minutes from cron.
require __DIR__ . '/lib.php';

foreach (glob(STORE . '/files/*/*', GLOB_ONLYDIR) ?: [] as $dir) {
    $meta = json_decode((string)@file_get_contents($dir . '/meta.json'), true);
    if (!is_array($meta) || !expired($meta)) {
        continue;
    }
    $handle = (string)($meta['handle'] ?? '');
    $size = (int)($meta['bytes'] ?? 0);
    remove_video($dir);
    if (valid_handle($handle) && is_file(handle_file($handle))) {
        change_handle($handle, fn(array $r) => array_merge($r, ['bytes' => max(0, ((int)($r['bytes'] ?? 0)) - $size)]));
    }
}
