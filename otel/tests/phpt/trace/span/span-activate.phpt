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

$scope = $tracer->startSpan('root');
$scope2 = $tracer->startSpan('child');
//assert($child->getContext()->getTraceId() === $root->getContext()->getTraceId());
//$child->end();
//$root->end();
?>
--EXPECTF--
