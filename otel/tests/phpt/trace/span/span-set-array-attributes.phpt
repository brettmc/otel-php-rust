--TEST--
Set array-valued attributes of each supported element type
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

$span = Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('root')->startSpan();
$span->setAttributes([
         'strings' => ['a', 'b'],
         'ints' => [1, 2],
         'floats' => [1.5, 2.5],
         'bools' => [true, false],
         'empty' => [],
     ])
     ->end();
var_dump(Memory::getSpans()[0]['attributes']);
?>
--EXPECTF--
array(4) {
  ["strings"]=>
  array(2) {
    [0]=>
    string(1) "a"
    [1]=>
    string(1) "b"
  }
  ["ints"]=>
  array(2) {
    [0]=>
    int(1)
    [1]=>
    int(2)
  }
  ["floats"]=>
  array(2) {
    [0]=>
    float(1.5)
    [1]=>
    float(2.5)
  }
  ["bools"]=>
  array(2) {
    [0]=>
    bool(true)
    [1]=>
    bool(false)
  }
}
