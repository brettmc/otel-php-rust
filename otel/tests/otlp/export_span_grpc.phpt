--TEST--
Export a span: grpc
--EXTENSIONS--
otel
--ENV--
OTEL_EXPORTER_OTLP_ENDPOINT=http://collector:4317
OTEL_EXPORTER_OTLP_PROTOCOL=grpc
--FILE--
<?php
use OpenTelemetry\API\Globals;

Globals::tracerProvider()
    ->getTracer('my_tracer')
    ->spanBuilder('root')
    ->setAttribute('exporter', 'grpc')
    ->startSpan()
    ->setStatus('Error', 'kaboom')
    ->end();
var_dump('done');
?>
--EXPECT--
string(4) "done"
