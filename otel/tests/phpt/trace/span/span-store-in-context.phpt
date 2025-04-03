--TEST--
Store span in context (contrib auto-instrumentation method)
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\Context\Context;
use OpenTelemetry\Context\Scope;

$tracer = Globals::tracerProvider()->getTracer('my_tracer');

$span = $tracer->spanBuilder('root')->startSpan();
$context = Context::getCurrent();
$ctx = $span->storeInContext($context);
var_dump($ctx);
Context::storage()->attach($ctx);

$scope = Context::storage()->scope();
assert($scope instanceof Scope);
$span = Span::fromContext($scope->context());
$span->end();
$scope->detach();
?>
--EXPECTF--
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "%s"

	Name        : root
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: %s
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset
