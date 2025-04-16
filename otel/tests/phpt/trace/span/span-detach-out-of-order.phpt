--TEST--
Detach spans out of order
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=none
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\Span;

$tracer = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url');

$root = $tracer->spanBuilder('root')->startSpan();
echo 'root span id: ' . $root->getContext()->getSpanId() . PHP_EOL;
$scope = $root->activate();
echo 'active span id: ' . Span::getCurrent()->getContext()->getSpanId() . PHP_EOL;
assert($root->getContext()->getSpanId() === Span::getCurrent()->getContext()->getSpanId()); //root is active span
$child = $tracer->spanBuilder('child')->startSpan();
echo 'child span id: ' . $child->getContext()->getSpanId() . PHP_EOL;
$childScope = $child->activate();
echo 'active span id: ' . Span::getCurrent()->getContext()->getSpanId() . PHP_EOL;
assert($child->getContext()->getSpanId() === Span::getCurrent()->getContext()->getSpanId()); //child is active span
$scope->detach();
echo 'active span id: ' . Span::getCurrent()->getContext()->getSpanId() . PHP_EOL;
//the active span here is undefined, because we detached out of order
$childScope->detach();
echo 'active span id: ' . Span::getCurrent()->getContext()->getSpanId() . PHP_EOL;
//still undefined behaviour
?>
--EXPECTF--
root span id: %s
active span id: %s
child span id: %s
active span id: %s
active span id: %s
active span id: %s
