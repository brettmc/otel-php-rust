--TEST--
Check if hooks receive modified return value
--EXTENSIONS--
otel
--SKIPIF--
<?php if (PHP_VERSION_ID >= 80000) echo 'skip requires php 7.x'; ?>
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.cli.enabled=1
otel.log.level=warn

--FILE--
<?php
\OpenTelemetry\Instrumentation\hook(null, 'helloWorld', null, function($object, array $params, int $return) { return ++$return;} );
\OpenTelemetry\Instrumentation\hook(null, 'helloWorld', null, function($object, array $params, int $return) { return ++$return;} );

function helloWorld(int $val): int {
    return $val;
}

var_dump(helloWorld(1));
?>
--EXPECT--
int(3)
