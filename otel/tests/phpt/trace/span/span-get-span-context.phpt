--TEST--
Get SpanContext from a Span
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--FILE--
<?php
use OpenTelemetry\API\Globals;

$span = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url')->spanBuilder('root')->startSpan();
$context = $span->getContext();
var_dump($context);
var_dump([
    'trace_id' => $context->getTraceId(),
    'span_id' => $context->getSpanId(),
    'is_valid' => $context->isValid(),
    'is_remote' => $context->isRemote(),
]);
?>
--EXPECTF--
object(OpenTelemetry\API\Trace\SpanContext)#1 (0) {
}
array(4) {
  ["trace_id"]=>
  string(32) "%s"
  ["span_id"]=>
  string(16) "%s"
  ["is_valid"]=>
  bool(true)
  ["is_remote"]=>
  bool(false)
}
