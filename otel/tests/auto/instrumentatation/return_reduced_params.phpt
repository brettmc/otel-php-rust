--TEST--
Check if pre hook can reduce then return $params
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
\OpenTelemetry\Instrumentation\hook(null, 'helloWorld', function($obj, array $params) {
    $params[1] = null;
    return $params;
});

function helloWorld($a = null, $b = null, $c = null) {
    var_dump($a, $b, $c);
}

helloWorld('a', 'b', 'c');
?>
--EXPECT--
string(1) "a"
NULL
string(1) "c"
