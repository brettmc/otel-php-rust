--TEST--
Test php7 functionality
--EXTENSIONS--
otel
--INI--
otel.log.level="trace"
otel.log.file="/dev/stdout"
--SKIPIF--
<?php if (PHP_VERSION_ID > 80000) die('skip requires PHP7'); ?>
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
function my_test() {
    echo "hello world\n";
}
my_test();
?>
--EXPECTF--
%A
[%s] [DEBUG] [%s] [ThreadId(%d)] PluginManager::init
[%s] [DEBUG] [%s] [ThreadId(%d)] registered fcall handlers
[%s] [DEBUG] [%s] [ThreadId(%d)] OpenTelemetry::RINIT
%A
[%s] [DEBUG] [%s] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ThreadStarted message= name=BatchSpanProcessor.ThreadStarted interval_in_millisecs=5000 max_export_batch_size=512 max_queue_size=2048
%A
[%s] [DEBUG] [%s] [ThreadId(%d)] OpenTelemetry::RSHUTDOWN
[%s] [DEBUG] [%s] [ThreadId(%d)] otel::request: event src/request.rs:%d message=RSHUTDOWN::not auto-closing root span...
[%s] [DEBUG] [%s] [ThreadId(%d)] otel::request: event src/request.rs:%d message=RSHUTDOWN::CONTEXT_STORAGE is empty :)
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
[%s] [DEBUG] [%s] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ThreadStopped message= name=BatchSpanProcessor.ThreadStopped