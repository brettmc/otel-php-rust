--TEST--
Activate a span, modify it via getCurrent()
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
use OpenTelemetry\API\Trace\Span;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$tracer = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url');

$root = $tracer->spanBuilder('root')->startSpan();
$root->setAttribute('is_root', true);
$scope = $root->activate();
$current = Span::getCurrent();
assert($current->getContext()->getTraceId() === $root->getContext()->getTraceId());
assert($current->getContext()->getSpanId() === $root->getContext()->getSpanId());
$scope->detach();
$current->updateName("updated");
$current->end();
var_dump(Memory::getSpans()[0]['name']);
var_dump(Memory::getSpans()[0]['attributes']);
?>
--EXPECT--
string(7) "updated"
array(1) {
  ["is_root"]=>
  bool(true)
}
