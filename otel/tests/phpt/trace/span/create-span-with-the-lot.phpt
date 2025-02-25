--TEST--
Create a span with all features
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--XFAIL--
Not all setters are implemented
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\StatusCode;

$span = Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('root')->startSpan();
$span->setStatus('Ok')
     ->setAttribute('foo', 'bar')
     ->setAttributes(['baz' => 'bat', 'num' => 2, 'pi' => 3.14159, 'a' => [1,2,3,4,5]])
     ->updateName('updated')
     ->recordException(new \Exception('kaboom'))
     ->addLink()
     ->addEvent()
     ->end();
?>
--EXPECTF--
Spans
Resource
	 ->  %A
Span #0
	Instrumentation Scope
%A
	Name        : updated
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: 0000000000000000
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Ok
	Attributes:
		 ->  foo: String(Owned("bar"))
		 ->  baz: String(Owned("bat"))
		 ->  num: I64(2)
		 ->  pi: F64(3.14159)
		 ->  a: Array(I64([1, 2, 3, 4, 5]))
    Events:
    Links:
    Exceptions: