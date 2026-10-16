--TEST--
Storage::scope() reads the current scope without detaching it
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=none
OTEL_LOGS_EXPORTER=none
OTEL_METRICS_EXPORTER=none
--INI--
otel.cli.enabled=1
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\Span;
use OpenTelemetry\Context\Context;

$tracer = Globals::tracerProvider()->getTracer('my_tracer');

$span = $tracer->spanBuilder('root')->startSpan();
$scope = $span->activate();
$spanId = $span->getContext()->getSpanId();

var_dump(Span::getCurrent()->getContext()->getSpanId() === $spanId);

// scope() is an accessor: the caller detaches through the Scope object it returns. Reading it
// must not deactivate the context, and must stay readable across repeated calls.
$first = Context::storage()->scope();
$second = Context::storage()->scope();
var_dump($first !== null, $second !== null);
var_dump(Span::getCurrent()->getContext()->getSpanId() === $spanId);

// The returned Scope refers to the same context that is still active.
var_dump(Span::fromContext($first->context())->getContext()->getSpanId() === $spanId);

$scope->detach();
var_dump(Span::getCurrent()->getContext()->getSpanId() === $spanId);

$span->end();
?>
--EXPECT--
bool(true)
bool(true)
bool(true)
bool(true)
bool(true)
bool(false)
