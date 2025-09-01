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

function myTestFunction() {
    var_dump('myTestFunction');
}

OpenTelemetry\Instrumentation\hook(
    null,
    'myTestFunction',
    function() {
        var_dump('pre hook');
    },
    function() {
        var_dump('post hook');
    }
);

myTestFunction();
$spans = Memory::getSpans();
?>
--EXPECT--
string(8) "pre hook"
string(14) "myTestFunction"
string(9) "post hook"