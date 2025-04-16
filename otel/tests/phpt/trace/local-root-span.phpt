--TEST--
Local root span
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=none
--INI--
otel.log.level="error"
otel.log.file="/dev/stdout"
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\LocalRootSpan;

$tracer = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url');

$root = $tracer->spanBuilder('root')->startSpan();
var_dump($root);
$scope = $root->activate();
var_dump($root);
var_dump(LocalRootSpan::current());
$root->end();
$scope->detach();

//there should be no local root span
var_dump(LocalRootSpan::current());
?>
--EXPECTF--
object(OpenTelemetry\API\Trace\Span)#3 (2) {
  ["context_id":"OpenTelemetry\API\Trace\Span":private]=>
  int(0)
  ["is_local_root":"OpenTelemetry\API\Trace\Span":private]=>
  bool(false)
}
object(OpenTelemetry\API\Trace\Span)#3 (2) {
  ["context_id":"OpenTelemetry\API\Trace\Span":private]=>
  int(1)
  ["is_local_root":"OpenTelemetry\API\Trace\Span":private]=>
  bool(true)
}
object(OpenTelemetry\API\Trace\Span)#4 (2) {
  ["context_id":"OpenTelemetry\API\Trace\Span":private]=>
  int(1)
  ["is_local_root":"OpenTelemetry\API\Trace\Span":private]=>
  bool(true)
}
object(OpenTelemetry\API\Trace\NonRecordingSpan)#4 (0) {
}