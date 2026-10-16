--TEST--
Set span kind, for every kind, via the SpanKind constants
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
use OpenTelemetry\API\Trace\SpanKind;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

// Every kind is asserted, by constant rather than by literal, so the constant and the
// handler's mapping are pinned to each other. A transposed pair is invisible otherwise:
// the span still exports, under the wrong kind.
$tracer = Globals::tracerProvider()->getTracer("my_tracer", '0.1', 'schema.url');
$kinds = [
    'KIND_INTERNAL' => SpanKind::KIND_INTERNAL,
    'KIND_CLIENT' => SpanKind::KIND_CLIENT,
    'KIND_SERVER' => SpanKind::KIND_SERVER,
    'KIND_PRODUCER' => SpanKind::KIND_PRODUCER,
    'KIND_CONSUMER' => SpanKind::KIND_CONSUMER,
];
foreach ($kinds as $name => $kind) {
    $tracer->spanBuilder($name)->setSpanKind($kind)->startSpan()->end();
}

foreach (Memory::getSpans() as $span) {
    printf("%s (%d) => %s\n", $span['name'], $kinds[$span['name']], $span['span_kind']);
}

// An unknown kind falls back to INTERNAL rather than aborting.
$tracer->spanBuilder('unknown')->setSpanKind(99)->startSpan()->end();
printf("unknown => %s\n", Memory::getSpans()[5]['span_kind']);
?>
--EXPECT--
KIND_INTERNAL (0) => Internal
KIND_CLIENT (1) => Client
KIND_SERVER (2) => Server
KIND_PRODUCER (3) => Producer
KIND_CONSUMER (4) => Consumer
unknown => Internal
