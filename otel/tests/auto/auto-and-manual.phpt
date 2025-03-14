--TEST--
Test auto + manual instrumentation together
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
OTEL_SPAN_PROCESSOR=batch
--INI--
;otel.log.level="trace"
;otel.log.file="/dev/stdout"
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

	Name        : i-was-renamed
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: 0000000000000000
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset
	Attributes:
		 ->  code.function.name: String(Owned("demoFunction"))
		 ->  code.file.path: String(Owned("%s"))
		 ->  code.line.number: I64(%d)
		 ->  my-attribute: String(Owned("my-value"))
		 ->  post.attribute: String(Owned("post.value"))