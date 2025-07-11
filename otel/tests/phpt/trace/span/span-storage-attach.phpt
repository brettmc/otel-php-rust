--TEST--
Store span in context then attach to storage
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.log.level="error"
otel.log.file="/dev/stdout"
otel.cli.enable=1
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\Span;
use OpenTelemetry\Context\Context;
use OpenTelemetry\Context\Scope;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$tracer = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url');
$span = $tracer->spanBuilder('root')->startSpan();
$ctx = $span->storeInContext(Context::getCurrent());
Context::storage()->attach($ctx);
unset($span);

$scope = Context::storage()->scope();
assert($scope instanceof Scope);
$span = Span::fromContext($scope->context());
$span->updateName('foo');
$span->end();
$scope->detach();
var_dump(Memory::getSpans()[0]['name']);
?>
--EXPECT--
string(3) "foo"
