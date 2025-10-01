--TEST--
Check if hooks are invoked for internal functions
--SKIPIF--
<?php if (PHP_VERSION_ID >= 80000) die('skip requires PHP 7.x'); ?>
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.cli.enabled=1
otel.log.level=warn
--FILE--
<?php
\OpenTelemetry\Instrumentation\hook(null, 'array_map', function() { var_dump('PRE');}, function() { var_dump('POST');} );

array_map('var_dump', ['HELLO']);
?>
--EXPECT--
string(3) "PRE"
string(5) "HELLO"
string(4) "POST"