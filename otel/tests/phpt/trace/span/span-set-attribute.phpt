--TEST--
Create a span with all features
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
$span->setAttribute('string', 'foo')
     ->setAttribute('int', 99)
     ->setAttribute('double', 1.5)
     ->setAttribute('bool_true', true)
     ->setAttribute('bool_false', false)
     ->setAttribute('array_int', [1,2,3])
     ->setAttribute('array_string', ['one','two','three'])
     ->end();
assert(Memory::count() === 1);
var_dump(Memory::getSpans()[0]['attributes']);
?>
--EXPECT--
array(7) {
  ["string"]=>
  string(3) "foo"
  ["int"]=>
  int(99)
  ["double"]=>
  float(1.5)
  ["bool_true"]=>
  bool(true)
  ["bool_false"]=>
  bool(false)
  ["array_int"]=>
  array(3) {
    [0]=>
    int(1)
    [1]=>
    int(2)
    [2]=>
    int(3)
  }
  ["array_string"]=>
  array(3) {
    [0]=>
    string(3) "one"
    [1]=>
    string(3) "two"
    [2]=>
    string(5) "three"
  }
}
