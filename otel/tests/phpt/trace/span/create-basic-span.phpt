--TEST--
Create a span
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\StatusCode;

$span = Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('root')->startSpan();
var_dump($span);
$span->setStatus('Ok')
     ->end();
?>
--XFAIL--
Hard-coded instrumentation scope
--EXPECTF--
object(OpenTelemetry\API\Trace\Span)#2 (0) {
}
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "my_tracer"

	Name        : root
	TraceId     : %s
	SpanId      : %s
	ParentSpanId: 0000000000000000
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Ok