--TEST--
Create a span with event
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$span = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url')->spanBuilder('root')->startSpan();
$span
    ->addEvent('my-event', ['foo' => 'bar'])
    ->addEvent('another-event')
    ->end();
$events = Memory::getSpans()[0]['events'];
assert(count($events) === 2);
var_dump($events);
?>
--EXPECTF--
array(2) {
  [0]=>
  array(3) {
    ["name"]=>
    string(8) "my-event"
    ["timestamp"]=>
    int(%d)
    ["attributes"]=>
    array(1) {
      ["foo"]=>
      string(3) "bar"
    }
  }
  [1]=>
  array(3) {
    ["name"]=>
    string(13) "another-event"
    ["timestamp"]=>
    int(%d)
    ["attributes"]=>
    array(0) {
    }
  }
}