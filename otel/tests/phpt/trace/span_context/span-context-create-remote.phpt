--TEST--
Create a remote SpanContext
--EXTENSIONS--
otel
--FILE--
<?php
use OpenTelemetry\API\Trace\SpanContext;

$context = SpanContext::createFromRemoteParent('2b4ef3412d587ce6e7880fb27a316b8c', '7480a670201f6340');
var_dump([
    'trace_id' => $context->getTraceId(),
    'span_id' => $context->getSpanId(),
    'is_valid' => $context->isValid(),
    'is_remote' => $context->isRemote(),
]);

// Respecting the incoming sampling decision is the main reason to call this at all, so the
// caller's flags have to reach the context rather than being assumed.
var_dump([
    'default'  => SpanContext::createFromRemoteParent('2b4ef3412d587ce6e7880fb27a316b8c', '7480a670201f6340')->isSampled(),
    'flags_0'  => SpanContext::createFromRemoteParent('2b4ef3412d587ce6e7880fb27a316b8c', '7480a670201f6340', 0)->isSampled(),
    'flags_1'  => SpanContext::createFromRemoteParent('2b4ef3412d587ce6e7880fb27a316b8c', '7480a670201f6340', 1)->isSampled(),
    'remote_1' => SpanContext::createFromRemoteParent('2b4ef3412d587ce6e7880fb27a316b8c', '7480a670201f6340', 1)->isRemote(),
]);
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
  bool(true)
}
array(4) {
  ["default"]=>
  bool(false)
  ["flags_0"]=>
  bool(false)
  ["flags_1"]=>
  bool(true)
  ["remote_1"]=>
  bool(true)
}