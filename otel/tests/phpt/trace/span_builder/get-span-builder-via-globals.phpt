--TEST--
Fetch a span builder from globals
--EXTENSIONS--
otel
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\StatusCode;

$provider = Globals::tracerProvider();
var_dump($provider);
$tracer = $provider->getTracer("my_tracer");
var_dump($tracer);
$builder = $tracer->spanBuilder('root');
var_dump($builder);
?>
--EXPECTF--
object(OpenTelemetry\API\Trace\TracerProvider)#1 (0) {
}
object(OpenTelemetry\API\Trace\Tracer)#2 (0) {
}
object(OpenTelemetry\API\Trace\SpanBuilder)#3 (0) {
}
