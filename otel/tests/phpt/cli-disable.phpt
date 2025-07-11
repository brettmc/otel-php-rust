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
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] OpenTelemetry::MINIT disabled for cli
int(0)
int(0)