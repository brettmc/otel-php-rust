--TEST--
Test auto + manual instrumentation together
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\API\Globals;

function demoFunction() {
    Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('manual-span')->startSpan()->end();
}

demoFunction();
?>
--EXPECTF--
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "%s"

	Name        : manual-span
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: %s
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset
Span #1
	Instrumentation Scope
		Name         : "php-auto-instrumentation"

	Name        : demoFunction
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: 0000000000000000
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset