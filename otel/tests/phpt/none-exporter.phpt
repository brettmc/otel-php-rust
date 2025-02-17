--TEST--
Configure no exporter
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=none
--FILE--
<?php
use OpenTelemetry\API\Globals;
Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('root')->startSpan()->end();
?>
--EXPECT--
