--TEST--
Create a span then update name
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=1
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$span = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url')->spanBuilder('root')->startSpan();
$span->updateName('updated')->end();

$span = Memory::getSpans()[0];
var_dump($span['name']);
?>
--EXPECT--
string(7) "updated"
