--TEST--
Create a span with event
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--XFAIL--
addEvent not implemented
--FILE--
<?php
use OpenTelemetry\API\Globals;

$span = Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('root')->startSpan();
$span->addEvent('my-event', ['foo' => 'bar'])
     ->end();
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
	Events:
	Event #0
    	Name      : my-event
    	Timestamp : %s
    	Attributes:
    		 ->  foo: String(Owned("bar"))