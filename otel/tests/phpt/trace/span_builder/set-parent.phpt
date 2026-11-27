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

// Upstream types this `ContextInterface|false|null`: null means "use the current context"
// and false means "no parent, start a new trace". Both are asserted with a span active, so
// that false parenting on it, as null does, is visible.
$root = $tracer->spanBuilder('active')->startSpan();
$scope = $root->activate();
$tracer->spanBuilder('child')->setParent(null)->startSpan()->end();
$tracer->spanBuilder('orphan')->setParent(false)->startSpan()->end();
$scope->detach();
$root->end();

$spans = [];
foreach (Memory::getSpans() as $span) {
    $spans[$span['name']] = $span;
}
var_dump($spans['child']['parent_span_id'] === $spans['active']['span_context']['span_id']);
var_dump($spans['orphan']['parent_span_id']);
var_dump($spans['orphan']['span_context']['trace_id'] !== $spans['active']['span_context']['trace_id']);

// Union types are reflected from 8.0. Below that phper declares the argument untyped.
$expected = PHP_VERSION_ID < 80000 ? null : 'OpenTelemetry\Context\ContextInterface|false|null';
foreach ([
    'OpenTelemetry\API\Trace\SpanBuilder',
    'OpenTelemetry\API\Trace\SpanBuilderInterface',
] as $class) {
    $type = (new ReflectionMethod($class, 'setParent'))->getParameters()[0]->getType();
    $actual = $type === null ? null : (string) $type;
    echo $class, ': ', $actual === $expected ? 'ok' : var_export($actual, true), PHP_EOL;
}

echo "DONE\n";
?>
--EXPECT--
string(16) "0000000000000000"
bool(true)
string(16) "0000000000000000"
bool(true)
OpenTelemetry\API\Trace\SpanBuilder: ok
OpenTelemetry\API\Trace\SpanBuilderInterface: ok
DONE
