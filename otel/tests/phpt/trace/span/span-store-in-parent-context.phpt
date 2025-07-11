--TEST--
Store span in context (context created from propagator)
--EXTENSIONS--
otel
--INI--
otel.cli.enable=1
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\Span;
use OpenTelemetry\Context\Context;
use OpenTelemetry\Context\Scope;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$headers = [
    'traceparent' => '00-e77388f01a826e2de7afdcd1eefc034e-d6ba64af4fa59b65-01',
];

$tracer = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url');

$parent = Globals::propagator()->extract($headers);
$span = $tracer->spanBuilder('root')->setParent($parent)->startSpan();
Context::storage()->attach($span->storeInContext($parent));
unset($span);

$scope = Context::storage()->scope();
$scope->detach();
$span = Span::fromContext($scope->context());
$span->end();

$span = Memory::getSpans()[0];
var_dump($span['span_context']['trace_id']);
var_dump($span['parent_span_id']);
?>
--EXPECT--
string(32) "e77388f01a826e2de7afdcd1eefc034e"
string(16) "d6ba64af4fa59b65"
