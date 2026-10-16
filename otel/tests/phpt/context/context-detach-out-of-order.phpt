--TEST--
Detaching a non-innermost scope leaves the innermost context active
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

$tracer = Globals::tracerProvider()->getTracer('my_tracer');

$outer = $tracer->spanBuilder('outer')->startSpan();
$outerScope = $outer->activate();
$inner = $tracer->spanBuilder('inner')->startSpan();
$innerScope = $inner->activate();

$innerId = $inner->getContext()->getSpanId();
var_dump(Span::getCurrent()->getContext()->getSpanId() === $innerId);

// Detaching out of order is undefined behaviour per the spec, and the only safe response is
// to do nothing. A ContextGuard restores whatever was active when it was created, so popping
// the inner guard here would deactivate a span nobody asked to detach -- and leave no way to
// ever deactivate it, since the inner scope's own detach would then find the wrong top.
$outerScope->detach();
var_dump(Span::getCurrent()->getContext()->getSpanId() === $innerId);

// Unwinding innermost-first still works afterwards.
$innerScope->detach();
var_dump(Span::getCurrent()->getContext()->getSpanId() === $outer->getContext()->getSpanId());

$outerScope->detach();
$inner->end();
$outer->end();
?>
--EXPECT--
bool(true)
bool(true)
bool(true)
