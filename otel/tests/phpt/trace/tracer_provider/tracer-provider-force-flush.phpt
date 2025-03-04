--TEST--
Tracer provider force flush
--EXTENSIONS--
otel
--FILE--
<?php
use OpenTelemetry\API\Globals;

$provider = Globals::tracerProvider();
var_dump($provider->forceFlush());
?>
--EXPECTF--
bool(true)