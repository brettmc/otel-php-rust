--TEST--
Check if post hook can modify return value
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
\OpenTelemetry\Instrumentation\hook(null, 'helloWorld', null, fn(): int => 17);

function helloWorld() {
    return 42;
}

var_dump(helloWorld());
?>
--EXPECT--
int(17)