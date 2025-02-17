--TEST--
Get tracer provider from Globals
--EXTENSIONS--
otel
--FILE--
<?php
use OpenTelemetry\API\Globals;

$provider = Globals::tracerProvider();
var_dump($provider);
?>
--EXPECTF--
object(OpenTelemetry\API\Trace\TracerProvider)#%d (0) {
}