--TEST--
Store span in context (context created from propagator)
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\Span;
use OpenTelemetry\Context\Context;
use OpenTelemetry\Context\Scope;

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
?>
--EXPECTF--
Spans
Resource
%A
Span #0
	Instrumentation Scope
%A
	Name        : root
	TraceId     : e77388f01a826e2de7afdcd1eefc034e
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: d6ba64af4fa59b65
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset