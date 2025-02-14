--TEST--
Get multiple tracer providers from Globals
--EXTENSIONS--
otel
--FILE--
<?php
use OpenTelemetry\API\Globals;

$one = Globals::tracerProvider();
var_dump($one);
$two = Globals::tracerProvider();
var_dump($two);
?>
--EXPECTF--
object(OpenTelemetry\API\Trace\TracerProvider)#1 (0) {
}
object(OpenTelemetry\API\Trace\TracerProvider)#2 (0) {
}