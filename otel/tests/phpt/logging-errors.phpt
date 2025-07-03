--TEST--
Test internal errors logged
--DESCRIPTION--
Invalid OTLP endpoint
--EXTENSIONS--
otel
--INI--
otel.log.level="warn"
otel.log.file="/dev/stdout"
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
?>
--EXPECTF--
[%s] [ERROR] [pid=%d] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ExportError message= name=BatchSpanProcessor.ExportError error=Operation failed: reqwest::Error { kind: Request, url: "http://does-not-exist:4318/v1/traces", source: hyper_util::client::legacy::Error(Connect, ConnectError("dns error", Custom { kind: Uncategorized, error: "%s" })) }
