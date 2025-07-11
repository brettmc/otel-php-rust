--TEST--
Test internal events logged
--EXTENSIONS--
otel
--INI--
otel.log.level="trace"
otel.log.file="/dev/stdout"
otel.cli.enable=1
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
Globals::tracerProvider()->forceFlush();
?>
--EXPECTF--
%A
[%s] [DEBUG] [%s] [ThreadId(%d)] OpenTelemetry::RINIT%A
[%s] [DEBUG] [%s] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ThreadStarted message= name=BatchSpanProcessor.ThreadStarted interval_in_millisecs=5000 max_export_batch_size=512 max_queue_size=2048
%A
[%s] [DEBUG] [%s] [ThreadId(%d)] opentelemetry_sdk: BatchSpanProcessor.ExportingDueToForceFlush message= name=BatchSpanProcessor.ExportingDueToForceFlush
Spans
Resource
%A
Span #0
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::trace::tracer_provider: event src/trace/tracer_provider.rs:%d message=OpenTelemetry tracer provider flush success
%A