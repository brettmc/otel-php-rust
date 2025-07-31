--TEST--
Export a span: http/protobuf
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=On
otel.log.level="error"
otel.log.file="/dev/stdout"
--ENV--
OTEL_EXPORTER_OTLP_ENDPOINT=http://collector:4318
OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf
OTEL_EXPORTER_OTLP_TIMEOUT=1500
OTEL_SERVICE_NAME=test-http-protobuf
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\StatusCode;

Globals::tracerProvider()
    ->getTracer('my_tracer', '0.1', 'schema.url')
    ->spanBuilder('root')
    ->setAttribute('exporter', 'http/protobuf')
    ->startSpan()
    ->setStatus(StatusCode::STATUS_ERROR, 'kaboom')
    ->end();
var_dump('done');
?>
--EXPECT--
string(4) "done"