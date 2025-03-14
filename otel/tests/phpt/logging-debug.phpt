--TEST--
Test internal errors logged
--EXTENSIONS--
otel
--INI--
otel.log.level="trace"
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
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::logging: event src/logging.rs:%d message=Logging::initialized level=debug path=/dev/stdout
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::trace::tracer_provider: event src/trace/tracer_provider.rs:38 message=span exporter=batch
[%s] [INFO] [pid=%d] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ThreadStarted message= name=BatchSpanProcessor.ThreadStarted interval_in_millisecs=5000 max_export_batch_size=512 max_queue_size=2048
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:%d message=RINIT::initializing
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:%d message=RINIT::sapi module name is: cli
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:%d message=RINIT::not auto-creating root span...
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::trace::span_builder: event src/trace/span_builder.rs:%d message=SpanBuilder::Starting span
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:%d message=RSHUTDOWN::maybe closing root span...
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel: event src/lib.rs:%d message=MSHUTDOWN::Shutting down OpenTelemetry exporter...
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ExportingDueToShutdown message= name=BatchSpanProcessor.ExportingDueToShutdown
Spans
Resource
%A
Span #0
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ThreadExiting message= name=BatchSpanProcessor.ThreadExiting reason=ShutdownRequested
[%s] [INFO] [pid=%d] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ThreadStopped message= name=BatchSpanProcessor.ThreadStopped
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel: event src/lib.rs:%d message=MSHUTDOWN::OpenTelemetry tracer provider shutdown success