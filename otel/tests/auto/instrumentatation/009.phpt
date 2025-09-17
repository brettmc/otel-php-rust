--TEST--
Check if pre hook can modify not provided arguments
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.cli.enabled=1
otel.log.level=trace
--XFAIL--
not implemented
--FILE--
<?php
\OpenTelemetry\Instrumentation\hook(null, 'helloWorld', fn() => [1 => 'b']);

function helloWorld($a = null, $b = null) {
    var_dump($a, $b);
}

helloWorld();
?>
--EXPECT--
NULL
string(1) "b"