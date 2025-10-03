--TEST--
Test excluded URL
--SKIPIF--
<?php
if (PHP_SAPI !== 'cli') {
    die('skip: Not running in CLI mode');
}
?>
--EXTENSIONS--
otel
--FILE--
<?php
include dirname(__DIR__) . '/run-server.php';

$options = [
    "http" => [
        "method" => "GET",
    ]
];

run_server('http/server-get.php', $options, '/health-check', 'OTEL_PHP_EXCLUDED_URLS=/health*,/foo/bar', 'trace');
?>
--EXPECTF--
==== Response ====
string(3) "GET"
string(13) "/health-check"
==== Server Output ====
%A
%sexcluded URL matched%s
%A