--TEST--
Export a span: grpc
--EXTENSIONS--
otel
--INI--
otel.cli.enable=On
otel.log.level="error"
otel.log.file="/dev/stdout"
--ENV--
OTEL_EXPORTER_OTLP_ENDPOINT=http://collector:4317
OTEL_EXPORTER_OTLP_PROTOCOL=grpc
OTEL_SERVICE_NAME=test-grpc
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\StatusCode;

Globals::tracerProvider()
    ->getTracer('my_tracer', '0.1', 'schema.url')
    ->spanBuilder('root')
    ->setAttribute('exporter', 'grpc')
    ->startSpan()
    ->setStatus(StatusCode::STATUS_ERROR, 'kaboom')
    ->end();
var_dump('done');
?>
--EXPECT--
string(4) "done"
