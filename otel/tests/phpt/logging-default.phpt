--TEST--
Test internal errors logged
--EXTENSIONS--
otel
--INI--
otel.log_level="warn"
--ENV--
OTEL_EXPORTER_OTLP_ENDPOINT=http://does-not-exist:4318
OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf
--FILE--
<?php
use OpenTelemetry\API\Globals;

Globals::tracerProvider()
    ->getTracer('my_tracer')
    ->spanBuilder('root')
    ->startSpan()
    ->end();
?>
--EXPECT--
Warning: PHP Shutdown: opentelemetry_sdk: BatchSpanProcessor.ExportError message= name=BatchSpanProcessor.ExportError error=Operation failed: reqwest::Error { kind: Request, url: "http://does-not-exist:4318/v1/traces", source: hyper_util::client::legacy::Error(Connect, ConnectError("dns error", Custom { kind: Uncategorized, error: "failed to lookup address information: Temporary failure in name resolution" })) } in Unknown on line 0