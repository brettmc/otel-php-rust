--TEST--
Get a tracer with instrumentation scope
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

$tracer = Globals::tracerProvider()->getTracer('test', '1.0', 'https://schemas.opentelemetry.io/1.30.0', ['a_string' => 'foo', 'a_bool' => true, 'a_int' => 3]);
$tracer->spanBuilder('test')->startSpan()->end();
$span = Memory::getSpans()[0];
var_dump($span['instrumentation_scope']);
?>
--EXPECT--
array(4) {
  ["name"]=>
  string(4) "test"
  ["version"]=>
  string(3) "1.0"
  ["schema_url"]=>
  string(39) "https://schemas.opentelemetry.io/1.30.0"
  ["attributes"]=>
  array(3) {
    ["a_string"]=>
    string(3) "foo"
    ["a_bool"]=>
    bool(true)
    ["a_int"]=>
    int(3)
  }
}