--TEST--
Create a span with attributes
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\StatusCode;
use OpenTelemetry\API\Trace\SpanContext;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$span = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url')->spanBuilder('root')->startSpan();
$span->setAttribute('foo', 'bar')
     ->setAttributes(['baz' => 'bat', 'num' => 2, 'pi' => 3.14159, 'a' => [1,2,3,4,5]])
     ->end();
var_dump(Memory::getSpans()[0]['attributes']);
?>
--EXPECTF--
array(5) {
  ["foo"]=>
  string(3) "bar"
  ["baz"]=>
  string(3) "bat"
  ["num"]=>
  int(2)
  ["pi"]=>
  float(3.14159)
  ["a"]=>
  array(5) {
    [0]=>
    int(1)
    [1]=>
    int(2)
    [2]=>
    int(3)
    [3]=>
    int(4)
    [4]=>
    int(5)
  }
}
