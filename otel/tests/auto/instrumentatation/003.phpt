--TEST--
Can call hook function
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
//use function OpenTelemetry\Instrumentation\hook;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

function helloWorld() {
    var_dump('HELLO');
}

OpenTelemetry\Instrumentation\hook(
    null,
    'helloWorld',
    function() {
        var_dump('PRE');
    },
    function() {
        var_dump('POST');
    }
);

helloWorld();
?>
--EXPECT--
string(3) "PRE"
string(5) "HELLO"
string(4) "POST"