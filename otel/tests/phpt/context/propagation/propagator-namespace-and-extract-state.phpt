--TEST--
The propagator is under API\Trace\Propagation and extract() returns a usable Context
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=1
--ENV--
OTEL_TRACES_EXPORTER=none
OTEL_LOGS_EXPORTER=none
OTEL_METRICS_EXPORTER=none
--FILE--
<?php
// Two things in one file.
//
// The class must be registered under upstream's `API\Trace\Propagation`, not the shorter
// `API\Propagation`: anyone following upstream's documentation names the former, and a
// wrong namespace is a class-not-found error.
//
// And an extracted Context must be fully usable, which is the one thing `extract()` is for.
// That needs both halves populated -- the `context_id` and the Rust state -- or every method
// reaching the underlying context (`with()`, `get()`, `activate()`) reports "not created by
// a Context or Storage factory method".
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\Propagation\TraceContextPropagator;
use OpenTelemetry\API\Trace\Span;
use OpenTelemetry\Context\Context;

var_dump(class_exists(TraceContextPropagator::class));
var_dump(class_exists('OpenTelemetry\API\Propagation\TraceContextPropagator'));
var_dump(Globals::propagator() instanceof TraceContextPropagator);

$headers = ['traceparent' => '00-e77388f01a826e2de7afdcd1eefc034e-d6ba64af4fa59b65-01'];
$ctx = Globals::propagator()->extract($headers);

// The extracted span is reachable, as it was before.
var_dump(Span::fromContext($ctx)->getContext()->getTraceId());

// ...and so is everything else a Context can do.
$key = Context::createKey('k');
$derived = $ctx->with($key, 'value');
var_dump($derived->get($key));
var_dump($ctx->get($key));

$scope = $derived->activate();
var_dump(Span::getCurrent()->getContext()->getSpanId());
var_dump(Context::getCurrent()->get($key));
var_dump($scope->detach());

// Round-trip: injecting the extracted context reproduces the header it came from.
$carrier = [];
Globals::propagator()->inject($carrier, null, $ctx);
var_dump($carrier['traceparent'] === $headers['traceparent']);
?>
--EXPECT--
bool(true)
bool(false)
bool(true)
string(32) "e77388f01a826e2de7afdcd1eefc034e"
string(5) "value"
NULL
string(16) "d6ba64af4fa59b65"
string(5) "value"
int(0)
bool(true)
