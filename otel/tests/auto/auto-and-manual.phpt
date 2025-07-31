--TEST--
Test auto + manual instrumentation together
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.cli.enabled=1
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

function demoFunction() {
    Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url')->spanBuilder('manual-span')->startSpan()->end();
}

demoFunction();
$spans = Memory::getSpans();
$one = $spans[0];
$two = $spans[1];

assert($one['name'] === 'manual-span');
assert($two['name'] === 'demo-function');
assert($one['span_context']['trace_id'] === $two['span_context']['trace_id']);
assert($one['parent_span_id'] === $two['span_context']['span_id']);
?>
--EXPECT--