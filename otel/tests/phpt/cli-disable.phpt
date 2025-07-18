--TEST--
Test disabled by default in CLI
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.log.level="debug"
otel.log.file="/dev/stdout"
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\SpanExporter\Memory;
$tracer = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url');
var_dump(Memory::count());
$tracer->spanBuilder('root')->startSpan()->end();
var_dump(Memory::count());
?>
--EXPECTF--
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel: event src/lib.rs:%d message=OpenTelemetry::MINIT disabled for cli
%A
int(0)%A
int(0)