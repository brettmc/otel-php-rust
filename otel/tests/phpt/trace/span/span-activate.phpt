--TEST--
Activate a span, create child span
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=1
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$tracer = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url');

$root = $tracer->spanBuilder('root')->startSpan();
$scope = $root->activate();
$child = $tracer->spanBuilder('child')->startSpan();
assert($child->getContext()->getTraceId() === $root->getContext()->getTraceId());
$child->end();
$root->end();
$scope->detach();

assert(Memory::count() === 2);
$child = Memory::getSpans()[0];
$root = Memory::getSpans()[1];
assert($child['name'] === 'child');
assert($root['name'] === 'root');
assert($child['span_context']['trace_id'] === $root['span_context']['trace_id']);
assert($child['parent_span_id'] === $root['span_context']['span_id']);
?>
--EXPECT--
