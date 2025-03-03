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
Notice: PHP Startup: opentelemetry_sdk: BatchSpanProcessor.ThreadStarted message= name=BatchSpanProcessor.ThreadStarted interval_in_millisecs=5000 max_export_batch_size=512 max_queue_size=2048 in Unknown on line 0
%A
Notice: PHP Shutdown: otel: event src/lib.rs:%i message=MSHUTDOWN::Shutting down OpenTelemetry exporter... in Unknown on line 0
%A
Notice: PHP Shutdown: opentelemetry_sdk: BatchSpanProcessor.ExportingDueToShutdown message= name=BatchSpanProcessor.ExportingDueToShutdown in Unknown on line 0
%A
Notice: PHP Shutdown: opentelemetry_sdk: BatchSpanProcessor.ThreadExiting message= name=BatchSpanProcessor.ThreadExiting reason=ShutdownRequested in Unknown on line 0
%A
Notice: PHP Shutdown: opentelemetry_sdk: BatchSpanProcessor.ThreadStopped message= name=BatchSpanProcessor.ThreadStopped in Unknown on line 0

Notice: PHP Shutdown: otel: event src/lib.rs:%i message=MSHUTDOWN::OpenTelemetry tracer provider shutdown success in Unknown on line 0