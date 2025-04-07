--TEST--
Activate a span, modify it via getCurrent()
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\Span;
$tracer = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url');

$root = $tracer->spanBuilder('root')->startSpan();
$root->setAttribute('is_root', true);
$scope = $root->activate();
$current = Span::getCurrent();
assert($current->getContext()->getTraceId() === $root->getContext()->getTraceId());
assert($current->getContext()->getSpanId() === $root->getContext()->getSpanId());
$scope->detach();
$current->updateName("updated");
$current->end();
?>
--EXPECTF--
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "%s"
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
	Attributes:
		 ->  is_root: Bool(true)