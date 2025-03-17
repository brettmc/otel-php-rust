--TEST--
Get a tracer with instrumentation scope
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\StatusCode;

$tracer = Globals::tracerProvider()->getTracer('test', '1.0', 'https://schemas.opentelemetry.io/1.30.0', ['a_string' => 'foo', 'a_bool' => true, 'a_int' => 3]);
$tracer->spanBuilder('test')->startSpan()->end();
?>
--EXPECTF--
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "test"
		Version  : "1.0"
		SchemaUrl: "https://schemas.opentelemetry.io/1.30.0"
		Scope Attributes:
			 ->  a_string: foo
			 ->  a_bool: true
			 ->  a_int: 3

	Name        : test
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: 0000000000000000
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset