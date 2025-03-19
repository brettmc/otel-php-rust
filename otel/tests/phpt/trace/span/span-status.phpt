--TEST--
Create a span with Error status + description
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\StatusCode;

$span = Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('root')->startSpan();
$span->setStatus(StatusCode::ERROR, 'kaboom')->end();
?>
--EXPECTF--
Spans
Resource
%A
Span #0
	Instrumentation Scope
%A

	Name        : root
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: 0000000000000000
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Error { description: "kaboom" }