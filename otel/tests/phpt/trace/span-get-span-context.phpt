--TEST--
Get SpanContext from a Span
--FILE--
<?php
use OpenTelemetry\Globals;

$span = Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('root')->startSpan();
$context = $span->getContext();
var_dump($context);
var_dump([
    'trace_id' => $context->getTraceId(),
    'span_id' => $context->getSpanId(),
    'is_valid' => $context->isValid(),
    'is_remote' => $context->isRemote(),
]);
$span->end();
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
Spans
Resource
%A
Span #0
	Instrumentation Scope
%A

	Name        : root
	TraceId     : %s
	SpanId      : %s
	ParentSpanId: 0000000000000000
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset