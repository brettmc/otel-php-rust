--TEST--
Store span in context (contrib auto-instrumentation method)
--EXTENSIONS--
otel
--INI--
otel.cli.enable=1
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
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
//get span back from context, mutate and end
$s = Span::fromContext($ctx);
$s->updateName("updated");
$s->end();
var_dump(Memory::getSpans()[0]['name']);
?>
--EXPECT--
string(7) "updated"