--TEST--
Activate a span, create child span
--EXTENSIONS--
otel
--FILE--
<?php
use OpenTelemetry\Globals;
use OpenTelemetry\API\Trace\StatusCode;
$tracer = Globals::tracerProvider()->getTracer('my_tracer');
$root = $tracer->spanBuilder('root')->startSpan();
/*$scope =*/ $root->activate();
$child = $tracer->spanBuilder('child')->startSpan();
$child->end();
$root->end();
?>
--EXPECTF--
