--TEST--
Create multiple root spans
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$tracer = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url');

$tracer->spanBuilder('root')->startSpan()->end();
$tracer->spanBuilder('two')->startSpan()->end();
$tracer->spanBuilder('three')->startSpan()->end();
assert(Memory::count() === 3);
$spans = Memory::getSpans();
foreach ($spans as $span) {
    assert($span['parent_span_id'] === '0000000000000000');
}
?>
--EXPECT--
