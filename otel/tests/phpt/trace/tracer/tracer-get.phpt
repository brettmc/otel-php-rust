--TEST--
Get tracer from tracer provider
--EXTENSIONS--
otel
--FILE--
<?php
use OpenTelemetry\API\Globals;

$tracer = Globals::tracerProvider()->getTracer("my_tracer");
var_dump($tracer);
?>
--EXPECTF--
object(OpenTelemetry\API\Trace\Tracer)#%d (0) {
}
