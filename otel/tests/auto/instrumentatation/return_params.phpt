--TEST--
Check if pre hook can return $params
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
\OpenTelemetry\Instrumentation\hook(null, 'helloWorld', fn($obj, array $params) => $params);

function helloWorld($a) {
    var_dump($a);
}

helloWorld('a');
?>
--EXPECT--
string(1) "a"
