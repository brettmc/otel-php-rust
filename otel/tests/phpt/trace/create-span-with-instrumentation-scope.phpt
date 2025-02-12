--TEST--
Create a span with instrumentation scope
--FILE--
<?php
use OpenTelemetry\Globals;
use OpenTelemetry\API\Trace\StatusCode;

$tracer = Globals::tracerProvider()->getTracer("test", "0.1", "https://schema.url", ["a_string" => "foo", "a_bool" => true, "a_int" => 3]);
$tracer->spanBuilder('test')->startSpan()->end();
?>
--EXPECTF--
FOOOO
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "change-me"

	Name        : test
	TraceId     : %s
	SpanId      : %s
	ParentSpanId: 0000000000000000
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset