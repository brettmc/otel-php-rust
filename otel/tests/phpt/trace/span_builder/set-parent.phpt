--TEST--
Call setParent on span builder
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
use OpenTelemetry\API\Trace\StatusCode;
use OpenTelemetry\Context\Context;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$builder = Globals::tracerProvider()->getTracer("my_tracer", '0.1', 'schema.url')->spanBuilder('root');
$builder->setParent(Context::getCurrent());
$builder->startSpan()->end();
var_dump(Memory::getSpans()[0]['parent_span_id']);
?>
--EXPECT--
string(16) "0000000000000000"
