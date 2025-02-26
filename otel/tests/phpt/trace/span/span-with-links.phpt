--TEST--
Create a span with link
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--XFAIL--
Panics on retrieve rust span context from PHP object
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\SpanContext;

$span = Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('root')->startSpan();
$span->addLink(SpanContext::create('2b4ef3412d587ce6e7880fb27a316b8c', '7480a670201f6340'));
$scope = $span->activate();
//add link after activate goes through a SpanRef
$span->addLink(SpanContext::create('fffff3412d587ce6e7880fb27a316b8c', 'ffffa670201f6340'));
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
    Links:
        FirstLink:
        SecondLink:
