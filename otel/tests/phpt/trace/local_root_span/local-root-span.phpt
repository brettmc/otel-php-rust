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

//initially, there is no local root span
var_dump(LocalRootSpan::current());
assert(LocalRootSpan::current()->getContext()->isValid() === false);

$root = $tracer->spanBuilder('root')->startSpan();
$scope = $root->activate();
$localRoot = LocalRootSpan::current();
assert(LocalRootSpan::current()->getContext()->getSpanId() === $root->getContext()->getSpanId());
var_dump(LocalRootSpan::current());
$root->end();
$scope->detach();

//there should be no local root span
var_dump(LocalRootSpan::current());
assert(LocalRootSpan::current()->getContext()->isValid() === false);
?>
--EXPECTF--
object(OpenTelemetry\API\Trace\NonRecordingSpan)#%d (0) {
}
object(OpenTelemetry\API\Trace\Span)#%d (2) {
  ["context_id":"OpenTelemetry\API\Trace\Span":private]=>
  int(1)
  ["is_local_root":"OpenTelemetry\API\Trace\Span":private]=>
  bool(true)
}
object(OpenTelemetry\API\Trace\NonRecordingSpan)#%d (0) {
}