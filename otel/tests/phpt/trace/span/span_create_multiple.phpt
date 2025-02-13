--TEST--
Create multiple root spans
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\Globals;

$tracer = Globals::tracerProvider()->getTracer('my_tracer');

$tracer->spanBuilder('root')->startSpan()->end();
$tracer->spanBuilder('two')->startSpan()->end();
$tracer->spanBuilder('three')->startSpan()->end();
?>
--EXPECTF--
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "change-me"
		Version  : "0.1"
		SchemaUrl: "http://my.schema.url"

	Name        : root
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: 0000000000000000
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset
Span #1
	Instrumentation Scope
		Name         : "change-me"
		Version  : "0.1"
		SchemaUrl: "http://my.schema.url"

	Name        : two
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: 0000000000000000
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset
Span #2
	Instrumentation Scope
		Name         : "change-me"
		Version  : "0.1"
		SchemaUrl: "http://my.schema.url"

	Name        : three
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: 0000000000000000
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset