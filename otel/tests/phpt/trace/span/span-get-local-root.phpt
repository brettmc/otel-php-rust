--TEST--
Activate some spans, get the root
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\Span;
$tracer = Globals::tracerProvider()->getTracer('my_tracer');

$root = $tracer->spanBuilder('root')->startSpan();
$scope = $root->activate();
$child = $tracer->spanBuilder('child')->startSpan();
$childScope = $child->activate();
$localRoot = Span::getLocalRoot();
assert($localRoot->getContext()->getSpanId() === $root->getContext()->getSpanId());
$child->end();
$childScope->detach();
$root->end();
$scope->detach();
?>
--EXPECTF--
%A