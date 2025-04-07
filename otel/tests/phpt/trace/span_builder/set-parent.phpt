--TEST--
Call setParent on span builder
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\StatusCode;
use OpenTelemetry\Context\Context;

$builder = Globals::tracerProvider()->getTracer("my_tracer", '0.1', 'schema.url')->spanBuilder('root');
$builder->setParent(Context::getCurrent());
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
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: 0000000000000000
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset
