--TEST--
Call setParent on span builder
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
use OpenTelemetry\API\Trace\StatusCode;
use OpenTelemetry\Context\Context;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$tracer = Globals::tracerProvider()->getTracer("my_tracer", '0.1', 'schema.url');
$builder = $tracer->spanBuilder('root');
$builder->setParent(Context::getCurrent());
$builder->startSpan()->end();
var_dump(Memory::getSpans()[0]['parent_span_id']);

// Upstream types this `ContextInterface|false|null`, where null means "use the current
// context" and false means "no parent, start a new trace". phper cannot express that union,
// so the hint is `?ContextInterface`: null is accepted -- asserted below, since it is the
// upstream-documented way to ask for the current context -- and `false` is a TypeError.
$root = $tracer->spanBuilder('active')->startSpan();
$scope = $root->activate();
$tracer->spanBuilder('child')->setParent(null)->startSpan()->end();
$scope->detach();
$root->end();

$spans = [];
foreach (Memory::getSpans() as $span) {
    $spans[$span['name']] = $span;
}
var_dump($spans['child']['parent_span_id'] === $spans['active']['span_context']['span_id']);
?>
--EXPECT--
string(16) "0000000000000000"
bool(true)
