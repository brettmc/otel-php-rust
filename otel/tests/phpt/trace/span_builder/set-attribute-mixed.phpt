--TEST--
SpanBuilder::setAttribute accepts the same value types as Span::setAttribute
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

// `$value` is `mixed` upstream, so a non-string attribute set on a *builder* must survive to
// the exported span -- not be dropped on the way, which is silent. The builder and the
// started span share `util::zval_to_key_value` so the two cannot disagree about what a value
// may be.
$tracer = Globals::tracerProvider()->getTracer('my_tracer');
$builder = $tracer->spanBuilder('span');
$builder
    ->setAttribute('string', 'str')
    ->setAttribute('int', 42)
    ->setAttribute('float', 1.5)
    ->setAttribute('bool', true)
    ->setAttribute('array', ['x', 'y']);
$span = $builder->startSpan();
// Set through the span as well: the two must accept the same types, or which object you
// happen to hold decides whether a float is allowed.
$span->setAttribute('int_via_span', 7);
$span->end();

var_dump(Memory::getSpans()[0]['attributes']);
?>
--EXPECT--
array(6) {
  ["string"]=>
  string(3) "str"
  ["int"]=>
  int(42)
  ["float"]=>
  float(1.5)
  ["bool"]=>
  bool(true)
  ["array"]=>
  array(2) {
    [0]=>
    string(1) "x"
    [1]=>
    string(1) "y"
  }
  ["int_via_span"]=>
  int(7)
}
