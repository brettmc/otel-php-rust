--TEST--
Create a span with Error status + description
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\StatusCode;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$span = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url')->spanBuilder('root')->startSpan();
$span->setStatus(StatusCode::STATUS_ERROR, 'kaboom')->end();
var_dump(Memory::getSpans()[0]['status']);
?>
--EXPECTF--
string(31) "Error { description: "kaboom" }"
