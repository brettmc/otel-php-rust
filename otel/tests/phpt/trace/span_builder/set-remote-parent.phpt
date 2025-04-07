--TEST--
Call setParent on SpanBuilder with remote span
--EXTENSIONS--
otel
--XFAIL--
todo implement Globals::propagator
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\StatusCode;
use OpenTelemetry\Context\Context;

$builder = Globals::tracerProvider()->getTracer("my_tracer", '0.1', 'schema.url')->spanBuilder('root');
$parent = Globals::propagator()->extract(['traceparent' => '00-e77388f01a826e2de7afdcd1eefc034e-d6ba64af4fa59b65-01']);
$builder->setParent($parent);
$builder->startSpan()->end();
?>
--EXPECTF--
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
