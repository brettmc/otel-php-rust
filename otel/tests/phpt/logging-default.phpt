--TEST--
Test internal errors logged
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
    ->getTracer('my_tracer')
    ->spanBuilder('root')
    ->startSpan()
    ->end();
?>
--EXPECTF--
[%s] [ERROR] [pid=%d] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ExportError message= name=BatchSpanProcessor.ExportError error=Operation failed: reqwest::Error { kind: Request, url: "http://does-not-exist:4318/v1/traces", source: hyper_util::client::legacy::Error(Connect, ConnectError("dns error", Custom { kind: Uncategorized, error: "failed to lookup address information: Temporary failure in name resolution" })) }
[%s] [WARN] [pid=%d] [ThreadId(%d)] otel::trace::tracer_provider: event src/trace/tracer_provider.rs:135 message=Failed to flush OpenTelemetry tracer provider: InternalFailure("errs: [Err(InternalFailure(\"Operation failed: reqwest::Error { kind: Request, url: \\\"http://does-not-exist:4318/v1/traces\\\", source: hyper_util::client::legacy::Error(Connect, ConnectError(\\\"dns error\\\", Custom { kind: Uncategorized, error: \\\"failed to lookup address information: Temporary failure in name resolution\\\" })) }\"))]")
