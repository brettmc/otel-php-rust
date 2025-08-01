--TEST--
disable otel via OTEL_DISABLED env var and try to create spans
--EXTENSIONS--
otel
--INI--
otel.log.level=debug
otel.cli.enabled=1
--ENV--
OTEL_DISABLED=true
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$tracer = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url');

$root = $tracer->spanBuilder('root')->startSpan();
$scope = $root->activate();
$root->end();
$scope->detach();

assert(Memory::count() === 0);
?>
--EXPECTF--
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::trace::tracer_provider: event src/trace/tracer_provider.rs:%d message=OpenTelemetry is disabled for this request, returning no-op tracer provider
%A