--TEST--
Check if hooks receive modified return value
--EXTENSIONS--
otel
--SKIPIF--
<?php if (version_compare(PHP_VERSION, '8.0.0', '<')) echo 'skip requires php 8.0+'; ?>
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.cli.enabled=1
otel.log.level=warn

--FILE--
<?php
\OpenTelemetry\Instrumentation\hook(null, 'helloWorld', null, fn(mixed $object, array $params, int $return): int => ++$return);
\OpenTelemetry\Instrumentation\hook(null, 'helloWorld', null, fn(mixed $object, array $params, int $return): int => ++$return);

function helloWorld(int $val): int {
    return $val;
}

var_dump(helloWorld(1));
?>
--EXPECT--
int(3)
