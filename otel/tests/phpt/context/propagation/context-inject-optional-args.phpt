--TEST--
inject() and extract() accept their optional arguments being omitted or null
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=1
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\Context\Context;

// Both methods take (carrier, setter|getter = null, context = null), so the ordinary
// call shape passes one argument. Reading the context slot unconditionally used to
// abort the process here — a panic at the extern "C" boundary cannot unwind into PHP,
// so it killed the request rather than raising. The existing inject tests all pass
// three arguments, which is why that went unnoticed.
$span = Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('root')->startSpan();
$scope = $span->activate();
$expected = '00-' . $span->getContext()->getTraceId() . '-' . $span->getContext()->getSpanId() . '-01';

// Omitted, explicitly null, and explicitly passed must all inject the active span.
$omitted = [];
Globals::propagator()->inject($omitted);
var_dump($omitted['traceparent'] === $expected);

$nulled = [];
Globals::propagator()->inject($nulled, null, null);
var_dump($nulled['traceparent'] === $expected);

$explicit = [];
Globals::propagator()->inject($explicit, null, Context::getCurrent());
// Loose comparison: keys are injected from a Rust HashMap, so their order is not
// stable and `===` would be asserting something the propagator never promised.
var_dump($explicit == $omitted && $nulled == $omitted);

// Same for extract: a null context means "use the current one", not a type error.
var_dump(Globals::propagator()->extract($omitted) instanceof Context);
var_dump(Globals::propagator()->extract($omitted, null, null) instanceof Context);

// Extracting an empty carrier yields a context with no remote parent, but it still
// inherits the active span, so a round-trip back out keeps the same traceparent.
$roundTripped = [];
Globals::propagator()->inject($roundTripped, null, Globals::propagator()->extract([]));
var_dump($roundTripped['traceparent'] === $expected);

$scope->detach();
$span->end();
?>
--EXPECT--
bool(true)
bool(true)
bool(true)
bool(true)
bool(true)
bool(true)
