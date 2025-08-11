<?php

/**
 * Run the PHP built-in web server in a shell, send an HTTP request to it.
 * The server output and response are captured and printed.
 */
function run_server(string $file, array $options, string $path = ''): void {
    $host = '127.0.0.1';
    $port = 8080;
    $docRoot = __DIR__;
    $router = $docRoot . '/' . $file;
    $server_log = sprintf('/tmp/server-%s.log', uniqid());

    // Start the PHP built-in web server
    $cmd = sprintf(
        '%s php %s -S %s:%d -t %s %s > %s 2>&1 & echo $!',
        'OTEL_TRACES_EXPORTER=console OTEL_SPAN_PROCESSOR=simple',
        '-d extension=otel.so', //-d otel.log.level=debug
        $host,
        $port,
        escapeshellarg($docRoot),
        escapeshellarg($router),
        $server_log
    );
    //echo $cmd;
    $pid = shell_exec($cmd);
    usleep(500000); // Wait for server to start

    // Make an HTTP request to the server
    $url = "http://$host:$port/$path";

    // Create context with options
    $context = stream_context_create($options);
    $response = file_get_contents($url, false, $context);

    // Kill the server
    exec("kill $pid");

    // Output response
    echo "==== Response ====\n";
    echo $response;

    echo "==== Server Output ====\n";
    echo file_get_contents($server_log);
}