<?php
// The page for a shared video, at /daisycap/<handle>/<video>. Unlisted: it
// asks search engines not to index it.
require __DIR__ . '/lib.php';

$h = (string)($_SERVER['SHARE_HANDLE'] ?? '');
$slug = (string)($_SERVER['SHARE_SLUG'] ?? '');
$ok = valid_handle($h) && valid_slug($slug);
$meta = $ok ? json_decode((string)@file_get_contents(video_dir($h, $slug) . '/meta.json'), true) : null;
if (is_array($meta) && expired($meta)) {
    $meta = null;
}

header('X-Robots-Tag: noindex');
header('Content-Type: text/html; charset=utf-8');
if (!is_array($meta)) {
    http_response_code(404);
    echo '<!doctype html><meta charset="utf-8"><title>Not found · DaisyCap</title>'
        . '<p style="font-family:system-ui;padding:2rem">No video here. <a href="https://jaeswift.xyz/daisycap">DaisyCap</a></p>';
    exit;
}

$title = trim((string)($meta['title'] ?? '')) ?: 'DaisyCap video';
$base = '/daisycap/' . $h . '/' . $slug;
$media = $base . '/' . $meta['file'];
$isGif = str_ends_with((string)$meta['file'], '.gif');
$poster = is_file(video_dir($h, $slug) . '/poster.jpg') ? $base . '/poster.jpg' : '';
?>
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="robots" content="noindex">
<title><?= e($title) ?> · DaisyCap</title>
<meta property="og:title" content="<?= e($title) ?>">
<meta property="og:site_name" content="DaisyCap">
<meta property="og:url" content="https://jaeswift.xyz<?= e($base) ?>">
<?php if ($poster !== ''): ?>
<meta property="og:image" content="https://jaeswift.xyz<?= e($poster) ?>">
<?php endif; ?>
<?php if (!$isGif): ?>
<meta property="og:video" content="https://jaeswift.xyz<?= e($media) ?>">
<meta property="og:video:type" content="video/mp4">
<?php endif; ?>
<meta name="twitter:card" content="summary_large_image">
<style>
  html, body { margin: 0; background: #0b0b0f; color: #f2f2f5; font-family: -apple-system, system-ui, sans-serif; }
  main { max-width: 960px; margin: 0 auto; padding: 32px 20px 48px; }
  .media { display: block; width: 100%; height: auto; border-radius: 10px; background: #000; }
  h1 { font-size: 22px; font-weight: 600; margin: 20px 0 6px; }
  p { color: #9a9aa6; margin: 0; font-size: 14px; }
  a { color: #7db8ff; }
</style>
</head>
<body>
<main>
<?php if ($isGif): ?>
  <img class="media" src="<?= e($media) ?>" alt="<?= e($title) ?>">
<?php else: ?>
  <video class="media" src="<?= e($media) ?>"<?= $poster !== '' ? ' poster="' . e($poster) . '"' : '' ?> controls playsinline preload="metadata"></video>
<?php endif; ?>
  <h1><?= e($title) ?></h1>
  <p>Shared by <?= e($h) ?> · <a href="https://jaeswift.xyz/daisycap">Made in DaisyCap</a></p>
</main>
</body>
</html>
