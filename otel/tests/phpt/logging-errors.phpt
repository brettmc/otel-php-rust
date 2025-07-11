--TEST--
Test internal errors logged
--DESCRIPTION--
Invalid OTLP endpoint
--EXTENSIONS--
otel
--INI--
otel.log.level="warn"
otel.log.file="/dev/stdout"
otel.cli.enable=1
--ENV--
OTEL_EXPORTER_OTLP_ENDPOINT=http://does-not-exist:4318
OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf
--FILE--
<?php
use OpenTelemetry\API\Globals;

Globals::tracerProvider()
    ->getTracer('my_tracer', '0.1', 'schema.url')
    ->spanBuilder('root')
    ->startSpan()
    ->end();
Globals::tracerProvider()->forceFlush();
?>
--EXPECTF--
[%s] [ERROR] [pid=%d] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ExportError message= name=BatchSpanProcessor.ExportError error=Operation failed: %s
%A