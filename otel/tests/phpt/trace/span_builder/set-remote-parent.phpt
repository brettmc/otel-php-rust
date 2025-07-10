--TEST--
Call setParent on SpanBuilder with remote span
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\StatusCode;
use OpenTelemetry\Context\Context;
use OpenTelemetry\Context\ContextInterface;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$builder = Globals::tracerProvider()->getTracer("my_tracer", '0.1', 'schema.url')->spanBuilder('root');
$carrier = ['traceparent' => '00-e77388f01a826e2de7afdcd1eefc034e-d6ba64af4fa59b65-01'];
$parent = Globals::propagator()->extract($carrier);
$span = OpenTelemetry\API\Trace\Span::fromContext($parent);
$context = $span->getContext();

$builder->setParent($parent);
$builder->startSpan()->end();

$span = Memory::getSpans()[0];
assert($span['parent_span_id'] === $context->getSpanId());
assert($span['span_context']['trace_id'] === $context->getTraceId());
?>
--EXPECT--
