--TEST--
Check if post hook can modify return value
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
\OpenTelemetry\Instrumentation\hook(null, 'helloWorld', null, function() { return 17;});

function helloWorld() {
    return 42;
}

var_dump(helloWorld());
?>
--EXPECT--
int(17)