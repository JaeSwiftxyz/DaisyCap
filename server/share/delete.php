<?php
// POST {"handle": "...", "slug": "..."} with the handle's token: remove a video.
require __DIR__ . '/lib.php';
require_post();

$in = json_decode((string)file_get_contents('php://input'), true);
$h = is_array($in) ? (string)($in['handle'] ?? '') : '';
$slug = is_array($in) ? (string)($in['slug'] ?? '') : '';
authed_handle($h);
if (!valid_slug($slug) || !is_dir(video_dir($h, $slug))) {
    fail(404, 'no such video');
}

$dir = video_dir($h, $slug);
$meta = json_decode((string)@file_get_contents($dir . '/meta.json'), true);
$size = (int)(is_array($meta) ? ($meta['bytes'] ?? 0) : 0);
remove_video($dir);
change_handle($h, fn(array $r) => array_merge($r, ['bytes' => max(0, ((int)($r['bytes'] ?? 0)) - $size)]));

json_out(200, ['ok' => true]);
