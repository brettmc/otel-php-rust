--TEST--
Autoinstrument a function
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.log.level="warn"
otel.log.file="/dev/stdout"
--FILE--
<?php
use OpenTelemetry\API\Trace\SpanExporter\Memory;

function demoFunction() {
    var_dump("demo_function");
}

demoFunction();
assert(Memory::count() === 1);
$span = Memory::getSpans()[0];
assert($span['name'] === 'demo-function');
assert($span['instrumentation_scope']['name'] === 'php-auto-instrumentation');
?>
--EXPECT--
string(13) "demo_function"