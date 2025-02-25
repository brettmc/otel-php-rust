--TEST--
Create a span with all features
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\StatusCode;

$span = Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('root')->startSpan();
$span->setAttribute('string', 'foo')
     ->setAttribute('int', 99)
     ->setAttribute('double', 1.5)
     ->setAttribute('bool_true', true)
     ->setAttribute('bool_false', false)
     ->setAttribute('array_int', [1,2,3])
     ->setAttribute('array_string', ['one','two','three'])
     ->end();
?>
--EXPECTF--
Spans
Resource
	 ->  %A
Span #0
	Instrumentation Scope
%A
	Name        : %s
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: 0000000000000000
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: %s
	Attributes:
		 ->  string: String(Owned("foo"))
		 ->  int: I64(99)
		 ->  double: F64(1.5)
		 ->  bool_true: Bool(true)
		 ->  bool_false: Bool(false)
		 ->  array_int: Array(I64([1, 2, 3]))
		 ->  array_string: Array(String([Owned("one"), Owned("two"), Owned("three")]))
