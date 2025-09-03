--TEST--
Check if hooks receive modified arguments
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
\OpenTelemetry\Instrumentation\hook(null, 'helloWorld', fn(mixed $object, array $params) => [++$params[0]]);
\OpenTelemetry\Instrumentation\hook(null, 'helloWorld', fn(mixed $object, array $params) => [++$params[0]]);

function helloWorld($a) {
    var_dump($a);
}

helloWorld(1);
?>
--EXPECT--
int(3)
