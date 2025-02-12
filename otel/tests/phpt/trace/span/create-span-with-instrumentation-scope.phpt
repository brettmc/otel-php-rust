--TEST--
Create a span with instrumentation scope
--EXTENSIONS--
otel
--FILE--
<?php
use OpenTelemetry\Globals;
use OpenTelemetry\API\Trace\StatusCode;

$tracer = Globals::tracerProvider()->getTracer('test', '1.0', 'https://schemas.opentelemetry.io/1.30.0', ['a_string' => 'foo', 'a_bool' => true, 'a_int' => 3]);
$tracer->spanBuilder('test')->startSpan()->end();
?>
--XFAIL--
Tracer with instrumentation scope not implemented
--EXPECTF--
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "test"
		Version  : "1.0"
		SchemaUrl: "https://schemas.opentelemetry.io/1.30.0"
		Attributes   : ???

	Name        : test
	TraceId     : %s
	SpanId      : %s
	ParentSpanId: 0000000000000000
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset