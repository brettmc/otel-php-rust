--TEST--
Test internal events logged
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
    ->getTracer('my_tracer', '0.1', 'schema.url')
    ->spanBuilder('root')
    ->startSpan()
    ->end();
?>
--EXPECTF--
%A
[%s] [DEBUG] [%s] [ThreadId(%d)] OpenTelemetry::RINIT
%A
[%s] [DEBUG] [%s] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ThreadStarted message= name=BatchSpanProcessor.ThreadStarted interval_in_millisecs=5000 max_export_batch_size=512 max_queue_size=2048
%A
[%s] [DEBUG] [%s] [ThreadId(%d)] OpenTelemetry::RSHUTDOWN
[%s] [DEBUG] [%s] [ThreadId(%d)] otel::request: event src/request.rs:%d message=RSHUTDOWN::not auto-closing root span...
[%s] [DEBUG] [%s] [ThreadId(%d)] otel::request: event src/request.rs:%d message=RSHUTDOWN::CONTEXT_STORAGE is empty :)
[%s] [DEBUG] [%s] [ThreadId(%d)] OpenTelemetry::MSHUTDOWN
[%s] [INFO] [%s] [ThreadId(%d)] otel::trace::tracer_provider: event src/trace/tracer_provider.rs:%d message=Shutting down TracerProvider for pid %d
[%s] [DEBUG] [%s] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ExportingDueToShutdown message= name=BatchSpanProcessor.ExportingDueToShutdown
Spans
Resource
%A
Span #0
%A
[%s] [DEBUG] [%s] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ThreadStopped message= name=BatchSpanProcessor.ThreadStopped