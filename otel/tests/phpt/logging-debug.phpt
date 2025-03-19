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
[%s] [DEBUG] [%s] [ThreadId(%d)] PluginManager::init
[%s] [DEBUG] [%s] [ThreadId(%d)] registered fcall handlers
[%s] [DEBUG] [%s] [ThreadId(%d)] OpenTelemetry::RINIT
%A
[%s] [INFO] [%s] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ThreadStarted message= name=BatchSpanProcessor.ThreadStarted interval_in_millisecs=5000 max_export_batch_size=512 max_queue_size=2048
%A
[%s] [DEBUG] [%s] [ThreadId(%d)] OpenTelemetry::RSHUTDOWN
[%s] [DEBUG] [%s] [ThreadId(%d)] otel::request: event src/request.rs:%d message=RSHUTDOWN::not auto-closing root span...
[%s] [DEBUG] [%s] [ThreadId(%d)] OpenTelemetry::MSHUTDOWN
[%s] [INFO] [%s] [ThreadId(%d)] otel::trace::tracer_provider: event src/trace/tracer_provider.rs:%d message=Flushing TracerProvider for pid %d
[%s] [DEBUG] [%s] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ExportingDueToForceFlush message= name=BatchSpanProcessor.ExportingDueToForceFlush
Spans
Resource
%A
Span #0
%A
[%s] [DEBUG] [%s] [ThreadId(%d)] otel::trace::tracer_provider: event src/trace/tracer_provider.rs:%d message=OpenTelemetry tracer provider flush success
[%s] [DEBUG] [%s] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ExportingDueToShutdown message= name=BatchSpanProcessor.ExportingDueToShutdown
[%s] [DEBUG] [%s] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ThreadExiting message= name=BatchSpanProcessor.ThreadExiting reason=ShutdownRequested
[%s] [INFO] [%s] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ThreadStopped message= name=BatchSpanProcessor.ThreadStopped