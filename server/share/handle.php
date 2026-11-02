<?php
// POST {"handle": "..."}: claim a handle. The first claim gets a token, which
// the app keeps. Later calls for that handle must send the same token.
require __DIR__ . '/lib.php';
require_post();

$in = json_decode((string)file_get_contents('php://input'), true);
$h = is_array($in) ? (string)($in['handle'] ?? '') : '';
if (!valid_handle($h)) {
    fail(400, 'handles are 3 to 30 letters, digits and single hyphens, and not reserved');
}

$fp = @fopen(handle_file($h), 'x'); // create-only: false when the handle exists
if ($fp !== false) {
    $token = bin2hex(random_bytes(24));
    fwrite($fp, json_encode([
        'token_hash' => password_hash($token, PASSWORD_DEFAULT),
        'created' => gmdate('c'),
        'bytes' => 0,
    ]));
    fclose($fp);
    json_out(200, ['ok' => true, 'handle' => $h, 'token' => $token, 'new' => true]);
}

$rec = json_decode((string)file_get_contents(handle_file($h)), true);
$token = bearer_token();
if ($token !== '' && is_array($rec) && password_verify($token, $rec['token_hash'] ?? '')) {
    json_out(200, ['ok' => true, 'handle' => $h, 'new' => false]);
}
fail(409, 'that handle is taken');
