--TEST--
Create a span
--EXTENSIONS--
otel
--ENV--
OTEL_EXPORTER_OTLP_ENDPOINT=http://collector:4318
OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf
OTEL_EXPORTER_OTLP_TIMEOUT=1500
--FILE--
<?php
use OpenTelemetry\Globals;

Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('root')->startSpan()->setStatus('Error', 'kaboom')->end();
var_dump('done');
?>
--EXPECT--
string(4) "done"