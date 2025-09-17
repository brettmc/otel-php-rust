--TEST--
Check if pre hook can expand then return $params
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.cli.enabled=1
otel.log.level=warn
--XFAIL--
expanding params does not work
--FILE--
<?php
\OpenTelemetry\Instrumentation\hook(null, 'helloWorld', function($obj, array $params) {
    $params[1] = 'b';
    return $params;
});

function helloWorld($a, $b = null) {
    var_dump($a);
    var_dump($b);
}

helloWorld('a');
?>
--EXPECT--
string(1) "a"
string(1) "b"
