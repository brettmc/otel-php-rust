--TEST--
Test internal errors logged
--EXTENSIONS--
otel
--INI--
otel.log.level="debug"
otel.log.file="/dev/stdout"
--ENV--
OTEL_TRACES_EXPORTER=console
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
[%s] [pid=%d] [ThreadId(%d)] otel::logging: event src/logging.rs:37 message=Logging::initialized level=debug path=/dev/stdout
[%s] [pid=%d] [ThreadId(%d)] otel::trace::tracer_provider: event src/trace/tracer_provider.rs:38 message=span exporter=batch
[%s] [pid=%d] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ThreadStarted message= name=BatchSpanProcessor.ThreadStarted interval_in_millisecs=5000 max_export_batch_size=512 max_queue_size=2048
[%s] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:28 message=RINIT::initializing
[%s] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:35 message=RINIT::sapi module name is: cli
[%s] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:38 message=RINIT::not auto-creating root span...
[%s] [pid=%d] [ThreadId(%d)] otel::trace::span_builder: event src/trace/span_builder.rs:57 message=SpanBuilder::Starting span
[%s] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:63 message=RSHUTDOWN::maybe closing root span...
[%s] [pid=%d] [ThreadId(%d)] otel: event src/lib.rs:100 message=MSHUTDOWN::Shutting down OpenTelemetry exporter...
[%s] [pid=%d] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ExportingDueToShutdown message= name=BatchSpanProcessor.ExportingDueToShutdown
Spans
Resource
%A
Span #0
%A
[%s] [pid=%d] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ThreadExiting message= name=BatchSpanProcessor.ThreadExiting reason=ShutdownRequested
[%s] [pid=%d] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ThreadStopped message= name=BatchSpanProcessor.ThreadStopped
[%s] [pid=%d] [ThreadId(%d)] otel: event src/lib.rs:104 message=MSHUTDOWN::OpenTelemetry tracer provider shutdown success