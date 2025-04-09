--TEST--
Call setParent on SpanBuilder with remote span
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\StatusCode;
use OpenTelemetry\Context\Context;
use OpenTelemetry\Context\ContextInterface;

$builder = Globals::tracerProvider()->getTracer("my_tracer", '0.1', 'schema.url')->spanBuilder('root');
$carrier = ['traceparent' => '00-e77388f01a826e2de7afdcd1eefc034e-d6ba64af4fa59b65-01'];
$parent = Globals::propagator()->extract($carrier);
$span = OpenTelemetry\API\Trace\Span::fromContext($parent);
$context = $span->getContext();

echo 'Trace ID: ' . $context->getTraceId() . PHP_EOL;
echo 'Span ID: ' . $context->getSpanId() . PHP_EOL;

$builder->setParent($parent);
$builder->startSpan()->end();
?>
--EXPECTF--
Trace ID: e77388f01a826e2de7afdcd1eefc034e
Span ID: d6ba64af4fa59b65
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "my_tracer"
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
