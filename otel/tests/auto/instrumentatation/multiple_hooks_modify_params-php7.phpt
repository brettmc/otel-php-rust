--TEST--
Check if hooks receive modified arguments
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
\OpenTelemetry\Instrumentation\hook(null, 'helloWorld', function($object, array $params) { return[++$params[0]]; });
\OpenTelemetry\Instrumentation\hook(null, 'helloWorld', function($object, array $params) { return[++$params[0]]; });

function helloWorld($a) {
    var_dump($a);
}

helloWorld(1);
?>
--EXPECT--
int(3)
