--TEST--
inject() and extract() take their context by name, skipping the setter or getter
--SKIPIF--
<?php
if (PHP_VERSION_ID < 80000) {
    // Named arguments are PHP 8.0 syntax, so the --FILE-- below is a parse error on 7.x.
    die("skip named arguments require PHP 8.0");
}
?>
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=1
--ENV--
OTEL_TRACES_EXPORTER=none
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\Context\Context;

// Skipping the setter or getter needs the declaration to record its default. Without one:
//   ArgumentCountError: Argument #2 ($setter) must be passed explicitly,
//                       because the default value is not known
$propagator = Globals::propagator();

// The context passed holds a span and the current one does not, so a call that ignored the
// argument would inject nothing.
$span = Globals::tracerProvider()->getTracer('t')->spanBuilder('s')->startSpan();
$spanId = $span->getContext()->getSpanId();
$context = $span->storeInContext(Context::getCurrent());

$injected = [];
$propagator->inject($injected, context: $context);
var_dump(str_contains($injected['traceparent'] ?? '', $spanId));

// extract() builds on the context it is given. With an empty carrier that leaves the span
// stored in it as the one a later inject() sees.
$extracted = $propagator->extract([], context: $context);
$injected = [];
$propagator->inject($injected, null, $extracted);
var_dump(str_contains($injected['traceparent'] ?? '', $spanId));

$span->end();
echo "DONE\n";
?>
--EXPECT--
bool(true)
bool(true)
DONE
