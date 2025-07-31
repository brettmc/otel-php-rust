--TEST--
Create a span and record exception
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
$span->recordException(new \Exception('kaboom'))
     ->end();
assert(Memory::count() === 1);
var_dump(Memory::getSpans()[0]['events']);
?>
--EXPECTF--
array(1) {
  [0]=>
  array(3) {
    ["name"]=>
    string(9) "exception"
    ["timestamp"]=>
    int(%d)
    ["attributes"]=>
    array(1) {
      ["exception.message"]=>
      string(6) "kaboom"
    }
  }
}
