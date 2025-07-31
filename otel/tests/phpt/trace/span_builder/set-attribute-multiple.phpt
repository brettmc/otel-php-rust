--TEST--
Call setAttribute multiple times on SpanBuilder
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
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$builder = Globals::tracerProvider()->getTracer("my_tracer", '0.1', 'schema.url')->spanBuilder('root');
$builder
    ->setAttribute('foo', 'bar')
    ->setAttribute('baz', 'bat');
$span = $builder->startSpan();
$span->end();
var_dump(Memory::getSpans()[0]['attributes']);
?>
--EXPECT--
array(2) {
  ["foo"]=>
  string(3) "bar"
  ["baz"]=>
  string(3) "bat"
}