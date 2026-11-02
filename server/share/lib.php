<?php
// Shared helpers for the DaisyCap share API and the video pages. Deployed to
// /opt/daisycap-share, outside the web root: nginx sends the public paths to
// these scripts (see daisycap-share.conf). The data lives in STORE.
declare(strict_types=1);

const STORE = '/var/lib/daisycap-share';
const MAX_VIDEO = 1073741824;        // 1 GB per video
const MAX_POSTER = 8388608;          // 8 MB per poster
const MAX_HANDLE_BYTES = 5368709120; // 5 GB per handle
const MIN_FREE_BYTES = 10737418240;  // keep 10 GB free on the disk
const LIFETIME_SECONDS = 3600;       // a shared video is deleted this long after upload
const RESERVED = ['api', 'daisycap', 'admin', 'www', 'wp-admin', 'static', 'assets', 'login', 'downloads'];

function json_out(int $code, array $body): never
{
    http_response_code($code);
    header('Content-Type: application/json; charset=utf-8');
    header('Cache-Control: no-store');
    echo json_encode($body, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE);
    exit;
}

function fail(int $code, string $message): never
{
    json_out($code, ['ok' => false, 'error' => $message]);
}

function require_post(): void
{
    if (($_SERVER['REQUEST_METHOD'] ?? '') !== 'POST') {
        fail(405, 'use POST');
    }
}

function e(string $s): string
{
    return htmlspecialchars($s, ENT_QUOTES | ENT_SUBSTITUTE, 'UTF-8');
}

function valid_handle(string $h): bool
{
    return preg_match('/^[a-z0-9][a-z0-9-]{1,28}[a-z0-9]$/', $h) === 1
        && !str_contains($h, '--')
        && !in_array($h, RESERVED, true);
}

function valid_slug(string $s): bool
{
    return preg_match('/^[a-z0-9][a-z0-9-]{2,80}$/', $s) === 1;
}

function bearer_token(): string
{
    $auth = $_SERVER['HTTP_AUTHORIZATION'] ?? '';
    return preg_match('/^Bearer ([A-Za-z0-9]{32,128})$/', $auth, $m) === 1 ? $m[1] : '';
}

function handle_file(string $h): string
{
    return STORE . '/handles/' . $h . '.json';
}

function video_dir(string $h, string $slug): string
{
    return STORE . '/files/' . $h . '/' . $slug;
}

/** The handle's record, if the request's bearer token is its token. Otherwise the request ends. */
function authed_handle(string $h): array
{
    if (!valid_handle($h)) {
        fail(400, 'bad handle');
    }
    $file = handle_file($h);
    $rec = is_file($file) ? json_decode((string)file_get_contents($file), true) : null;
    if (!is_array($rec)) {
        fail(404, 'no such handle');
    }
    $token = bearer_token();
    if ($token === '' || !password_verify($token, $rec['token_hash'] ?? '')) {
        fail(401, 'wrong or missing token');
    }
    return $rec;
}

/** Change a handle's record under a lock. Returns the new record. */
function change_handle(string $h, callable $change): array
{
    $fp = fopen(handle_file($h), 'c+');
    flock($fp, LOCK_EX);
    $rec = json_decode((string)stream_get_contents($fp), true);
    $rec = $change(is_array($rec) ? $rec : []);
    ftruncate($fp, 0);
    rewind($fp);
    fwrite($fp, json_encode($rec));
    fflush($fp);
    flock($fp, LOCK_UN);
    fclose($fp);
    return $rec;
}

/** Whether a video's upload is older than its lifetime. */
function expired(array $meta): bool
{
    return time() - strtotime((string)($meta['created'] ?? '')) > LIFETIME_SECONDS;
}

/** A readable, unguessable video name: the title, then six random hex digits. */
function make_slug(string $title): string
{
    $base = strtolower($title);
    $base = preg_replace('/[^a-z0-9]+/', '-', $base) ?? '';
    $base = trim(substr($base, 0, 40), '-');
    return ($base !== '' ? $base : 'video') . '-' . bin2hex(random_bytes(3));
}

/** Stream the request body to $dst, refusing more than $limit bytes. Returns the size. */
function copy_body($in, string $dst, int $limit): int
{
    $out = fopen($dst, 'wb');
    $total = 0;
    while (!feof($in)) {
        $chunk = fread($in, 1048576);
        if ($chunk === false || $chunk === '') {
            continue;
        }
        $total += strlen($chunk);
        if ($total > $limit) {
            fclose($out);
            @unlink($dst);
            fail(413, 'too big: videos are up to 1 GB, posters up to 8 MB');
        }
        fwrite($out, $chunk);
    }
    fclose($out);
    return $total;
}

function remove_video(string $dir): void
{
    foreach (glob($dir . '/*') ?: [] as $f) {
        @unlink($f);
    }
    @rmdir($dir);
}
