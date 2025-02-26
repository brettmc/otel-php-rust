--TEST--
Create a span and record exception
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\API\Globals;

$span = Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('root')->startSpan();
$span->recordException(new \Exception('kaboom'))
     ->end();
?>
--EXPECTF--
Spans
Resource
	 ->  %A
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
	Status: Unset
	Events:
	Event #0
	Name      : exception
	Timestamp : %s
	Attributes:
		 ->  exception.message: String(Owned("kaboom"))
