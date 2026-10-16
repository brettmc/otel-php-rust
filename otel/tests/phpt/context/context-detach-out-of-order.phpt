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
use OpenTelemetry\Context\ScopeInterface;

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
// The refusal is reported, not silent: MISMATCH is how the caller learns its detach did
// nothing. var_dump, not assert, because the failure mode worth catching is `detach()`
// returning NULL -- which `assert(!$x)` would accept just as happily as 0.
var_dump($outerScope->detach() === ScopeInterface::MISMATCH);
var_dump(Span::getCurrent()->getContext()->getSpanId() === $innerId);

// Unwinding innermost-first still works afterwards, and reports success as zero.
var_dump($innerScope->detach() === 0);
var_dump(Span::getCurrent()->getContext()->getSpanId() === $outer->getContext()->getSpanId());

var_dump($outerScope->detach() === 0);
// A scope that is no longer on the stack at all is DETACHED, which is a different condition
// from the MISMATCH above -- the whole point of returning a bitmask rather than a bool.
var_dump($outerScope->detach() === ScopeInterface::DETACHED);
$inner->end();
$outer->end();
?>
--EXPECT--
bool(true)
bool(true)
bool(true)
bool(true)
bool(true)
bool(true)
bool(true)
