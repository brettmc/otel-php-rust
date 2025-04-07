--TEST--
Create a span with attributes
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\StatusCode;
use OpenTelemetry\API\Trace\SpanContext;

$span = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url')->spanBuilder('root')->startSpan();
$span->setAttribute('foo', 'bar')
     ->setAttributes(['baz' => 'bat', 'num' => 2, 'pi' => 3.14159, 'a' => [1,2,3,4,5]])
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
	Kind        : %s
	Start time: %s
	End time: %s
	Status: Unset
	Attributes:
		 ->  foo: String(Owned("bar"))
		 ->  baz: String(Owned("bat"))
		 ->  num: I64(2)
		 ->  pi: F64(3.14159)
		 ->  a: Array(I64([1, 2, 3, 4, 5]))
