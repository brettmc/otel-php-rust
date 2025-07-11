--TEST--
Get tracer from tracer provider
--EXTENSIONS--
otel
--FILE--
<?php
use OpenTelemetry\API\Globals;

var_dump(Globals::tracerProvider()->getTracer("my_tracer", '0.1', 'schema.url'));
var_dump(Globals::tracerProvider()->getTracer("my_tracer", '0.1'));
var_dump(Globals::tracerProvider()->getTracer("my_tracer"));
var_dump(Globals::tracerProvider()->getTracer("my_tracer", '0.1', 'schema.url', ['one' => 1]));
?>
--EXPECTF--
object(OpenTelemetry\API\Trace\Tracer)#%d (0) {
}
object(OpenTelemetry\API\Trace\Tracer)#%d (0) {
}
object(OpenTelemetry\API\Trace\Tracer)#%d (0) {
}
object(OpenTelemetry\API\Trace\Tracer)#%d (0) {
}
