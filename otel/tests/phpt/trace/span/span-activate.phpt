--TEST--
Activate a span, create child span
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--XFAIL--
Should not use startAndActivate span
--FILE--
<?php
use OpenTelemetry\API\Globals;
$tracer = Globals::tracerProvider()->getTracer('my_tracer');

$scope = $tracer->startAndActivateSpan('root');
$scope2 = $tracer->startAndActivateSpan('child');
//assert($child->getContext()->getTraceId() === $root->getContext()->getTraceId());
//$child->end();
//$root->end();
?>
--EXPECTF--
