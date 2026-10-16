--TEST--
Create a SpanContext
--EXTENSIONS--
otel
--FILE--
<?php
use OpenTelemetry\API\Trace\SpanContext;

$context = SpanContext::create('2b4ef3412d587ce6e7880fb27a316b8c', '7480a670201f6340');
var_dump([
    'trace_id' => $context->getTraceId(),
    'span_id' => $context->getSpanId(),
    'is_valid' => $context->isValid(),
    'is_remote' => $context->isRemote(),
]);

// The caller's trace flags must reach the context rather than being assumed: upstream
// defaults to TraceFlags::DEFAULT (0, not sampled), and an explicit 0 must stay unsampled.
var_dump([
    'default' => SpanContext::create('2b4ef3412d587ce6e7880fb27a316b8c', '7480a670201f6340')->isSampled(),
    'flags_0' => SpanContext::create('2b4ef3412d587ce6e7880fb27a316b8c', '7480a670201f6340', 0)->isSampled(),
    'flags_1' => SpanContext::create('2b4ef3412d587ce6e7880fb27a316b8c', '7480a670201f6340', 1)->isSampled(),
]);

// A bad span id must report the span id as the problem, not the trace id.
try {
    SpanContext::create('2b4ef3412d587ce6e7880fb27a316b8c', 'nonsense');
} catch (Throwable $e) {
    echo get_class($e), ': ', $e->getMessage(), PHP_EOL;
}
?>
--EXPECTF--
array(4) {
  ["trace_id"]=>
  string(32) "2b4ef3412d587ce6e7880fb27a316b8c"
  ["span_id"]=>
  string(16) "7480a670201f6340"
  ["is_valid"]=>
  bool(true)
  ["is_remote"]=>
  bool(false)
}
array(3) {
  ["default"]=>
  bool(false)
  ["flags_0"]=>
  bool(false)
  ["flags_1"]=>
  bool(true)
}
%s: Invalid span id format
