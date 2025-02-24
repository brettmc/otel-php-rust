--TEST--
Activate a span, create child span
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\API\Globals;
$tracer = Globals::tracerProvider()->getTracer('my_tracer');

$root = $tracer->spanBuilder('root')->startSpan();
$scope = $root->activate();
$child = $tracer->spanBuilder('child')->startSpan();
//var_dump($child->getContext()->getTraceId());
//var_dump($root->getContext()->getTraceId());
//assert($child->getContext()->getTraceId() === $root->getContext()->getTraceId());
$child->end();
$root->end();
$scope->detach();
//TODO make some assertions to confirm it worked
?>
--EXPECTF--
