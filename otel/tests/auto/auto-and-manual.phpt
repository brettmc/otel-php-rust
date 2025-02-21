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
    var_dump("demo_function");
    Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('manual-span')->startSpan()->end();
}

demoFunction();
?>
--EXPECTF--
string(13) "demo_function"
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "php-auto-instrumentation"

	Name        : <global>::demoFunction
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: %s
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset