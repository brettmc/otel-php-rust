--TEST--
Store span in context (contrib auto-instrumentation method)
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\Span;
use OpenTelemetry\Context\Context;
use OpenTelemetry\Context\Scope;

$tracer = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url');

$span = $tracer->spanBuilder('root')->startSpan();
$ctx = $span->storeInContext(Context::getCurrent());
//get span back from context, mutate and end
$s = Span::fromContext($ctx);
$s->updateName("updated");
$s->end();
?>
--EXPECTF--
Spans
Resource
%A
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
	Status: Unset