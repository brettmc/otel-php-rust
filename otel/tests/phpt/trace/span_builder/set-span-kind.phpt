--TEST--
Set span kind
--EXTENSIONS--
otel
--INI--
otel.cli.enable=1
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\StatusCode;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$builder = Globals::tracerProvider()->getTracer("my_tracer", '0.1', 'schema.url')->spanBuilder('root');
$builder->setSpanKind(3); //producer
$builder->startSpan()->end();
var_dump(Memory::getSpans()[0]['span_kind']);
?>
--EXPECT--
string(8) "Producer"
