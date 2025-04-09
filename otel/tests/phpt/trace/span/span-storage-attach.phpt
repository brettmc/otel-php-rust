--TEST--
Store span in context then attach to storage
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--INI--
otel.log.level="error"
otel.log.file="/dev/stdout"
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\Span;
use OpenTelemetry\Context\Context;
use OpenTelemetry\Context\Scope;

$tracer = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url');
$span = $tracer->spanBuilder('root')->startSpan();
$ctx = $span->storeInContext(Context::getCurrent());
Context::storage()->attach($ctx);
unset($span);

$scope = Context::storage()->scope();
assert($scope instanceof Scope);
$span = Span::fromContext($scope->context());
$span->updateName('foo');
$span->end();
$scope->detach();
?>
--EXPECTF--
Spans
Resource
%A
Span #0
	Instrumentation Scope
%A
	Name        : foo
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: %s
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset
