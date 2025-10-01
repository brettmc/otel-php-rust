--TEST--
Check if pre hook can modify not provided arguments
--EXTENSIONS--
otel
--SKIPIF--
<?php if (PHP_VERSION_ID >= 80000) echo 'skip requires php 7'; ?>
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.cli.enabled=1
otel.log.level=warn
--FILE--
<?php
\OpenTelemetry\Instrumentation\hook(null, 'helloWorld', function() { return [1 => 'b'];} );

function helloWorld($a = null, $b = null) {
    var_dump($a, $b);
}

helloWorld();
?>
--EXPECT--
NULL
string(1) "b"