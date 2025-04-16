--TEST--
Non-recording span context
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=none
--FILE--
<?php
use OpenTelemetry\API\Trace\LocalRootSpan;
use OpenTelemetry\API\Trace\NonRecordingSpan;

$span = LocalRootSpan::current();
var_dump($span);
$context = $span->getContext();
var_dump([
    'trace_id' => $context->getTraceId(),
    'span_id' => $context->getSpanId(),
    'is_valid' => $context->isValid(),
    'is_remote' => $context->isRemote(),
    'is_sampled' => $context->isSampled(),
]);
?>
--EXPECTF--
object(OpenTelemetry\API\Trace\NonRecordingSpan)#1 (0) {
}
array(5) {
  ["trace_id"]=>
  string(32) "00000000000000000000000000000000"
  ["span_id"]=>
  string(16) "0000000000000000"
  ["is_valid"]=>
  bool(false)
  ["is_remote"]=>
  bool(false)
  ["is_sampled"]=>
  bool(false)
}