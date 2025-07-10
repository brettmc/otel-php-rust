--TEST--
Create a span with link
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
OTEL_SPAN_PROCESSOR=simple
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\SpanContext;

$span = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url')->spanBuilder('root')->startSpan();
$ctx_one = SpanContext::create('2b4ef3412d587ce6e7880fb27a316b8c', '7480a670201f6340');
$span->addLink($ctx_one);
$scope = $span->activate();
//add link after activate goes through a SpanRef (TODO, not implemented in opentelemetry-rust)
$ctx_two = SpanContext::create('fffff3412d587ce6e7880fb27a316b8c', 'ffffa670201f6340');
$span->addLink($ctx_two);
$span->end();
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
	Links:
	Link #0
	TraceId: 2b4ef3412d587ce6e7880fb27a316b8c
	SpanId : 7480a670201f6340
