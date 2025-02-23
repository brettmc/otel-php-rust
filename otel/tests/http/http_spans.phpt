--TEST--
PHP Built-in Webserver Test
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
$host = '127.0.0.1';
$port = 8080;
$docRoot = __DIR__;
$router = $docRoot . '/server.php';

// Start the PHP built-in web server
$cmd = sprintf(
    '%s php %s -S %s:%d -t %s %s > /dev/null 2>&1 & echo $!',
    'OTEL_TRACES_EXPORTER=console',
    '-d extension=otel.so',
    $host,
    $port,
    escapeshellarg($docRoot),
    escapeshellarg($router)
);
//var_dump($cmd);
$pid = shell_exec($cmd);
usleep(500000); // Wait for server to start

// Make an HTTP request to the server
$url = "http://$host:$port/";
$response = file_get_contents($url);

// Kill the server
exec("kill $pid");

// Output response
echo $response;
?>
--EXPECT--
Hello, World!
SOME_SPAN
