<?php
// POST with the file as the raw body. Headers:
//   Authorization: Bearer <token>     X-Share-Handle: <handle>
//   X-Share-Kind: video | poster      X-Share-Title: <percent-encoded title>
//   X-Share-Width, X-Share-Height, X-Share-Duration-Ms   (videos)
//   X-Share-Slug                      (posters: the video they belong to)
require __DIR__ . '/lib.php';
require_post();

$h = (string)($_SERVER['HTTP_X_SHARE_HANDLE'] ?? '');
$rec = authed_handle($h);
$kind = (string)($_SERVER['HTTP_X_SHARE_KIND'] ?? 'video');
$in = fopen('php://input', 'rb');

if ($kind === 'poster') {
    $slug = (string)($_SERVER['HTTP_X_SHARE_SLUG'] ?? '');
    $dir = video_dir($h, $slug);
    if (!valid_slug($slug) || !is_dir($dir)) {
        fail(404, 'no such video');
    }
    $tmp = STORE . '/tmp/' . bin2hex(random_bytes(8));
    copy_body($in, $tmp, MAX_POSTER);
    $fh = fopen($tmp, 'rb');
    $head = (string)fread($fh, 3);
    fclose($fh);
    if ($head !== "\xFF\xD8\xFF") {
        @unlink($tmp);
        fail(415, 'the poster must be a JPEG');
    }
    rename($tmp, $dir . '/poster.jpg');
    json_out(200, ['ok' => true]);
}

if ($kind !== 'video') {
    fail(400, 'unknown kind');
}
if (disk_free_space(STORE) < MIN_FREE_BYTES) {
    fail(507, 'the server is nearly out of space; try again later');
}

$tmp = STORE . '/tmp/' . bin2hex(random_bytes(8));
$size = copy_body($in, $tmp, MAX_VIDEO);
if (((int)($rec['bytes'] ?? 0)) + $size > MAX_HANDLE_BYTES) {
    @unlink($tmp);
    fail(507, 'this handle is full: remove a video first');
}
$fh = fopen($tmp, 'rb');
$head = (string)fread($fh, 12);
fclose($fh);
if (substr($head, 4, 4) === 'ftyp') {
    $file = 'video.mp4';   // MP4 and MOV share this container
} elseif (substr($head, 0, 3) === 'GIF') {
    $file = 'video.gif';
} else {
    @unlink($tmp);
    fail(415, 'send an MP4, MOV or GIF');
}

$title = mb_substr(rawurldecode((string)($_SERVER['HTTP_X_SHARE_TITLE'] ?? '')), 0, 200);
$slug = make_slug($title);
while (is_dir(video_dir($h, $slug))) {
    $slug = make_slug($title);
}
$dir = video_dir($h, $slug);
mkdir($dir, 0755, true);
rename($tmp, $dir . '/' . $file);
file_put_contents($dir . '/meta.json', json_encode([
    'handle' => $h,
    'title' => $title,
    'file' => $file,
    'bytes' => $size,
    'width' => (int)($_SERVER['HTTP_X_SHARE_WIDTH'] ?? 0),
    'height' => (int)($_SERVER['HTTP_X_SHARE_HEIGHT'] ?? 0),
    'duration_ms' => (int)($_SERVER['HTTP_X_SHARE_DURATION_MS'] ?? 0),
    'created' => gmdate('c'),
]));
change_handle($h, fn(array $r) => array_merge($r, ['bytes' => ((int)($r['bytes'] ?? 0)) + $size]));

json_out(200, [
    'ok' => true,
    'slug' => $slug,
    'url' => 'https://jaeswift.xyz/daisycap/' . $h . '/' . $slug,
]);
