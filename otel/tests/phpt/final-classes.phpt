--TEST--
Classes that upstream declares final are final
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=1
--ENV--
OTEL_TRACES_EXPORTER=none
--FILE--
<?php
// Only classes exposed under upstream's own name. `Storage` is upstream's final
// `ContextStorage` under another name, so it is left open.
foreach ([
    'OpenTelemetry\API\Globals',
    'OpenTelemetry\API\Trace\NonRecordingSpan',
    'OpenTelemetry\API\Trace\Propagation\TraceContextPropagator',
    'OpenTelemetry\API\Trace\SpanContext',
    'OpenTelemetry\Context\Context',
    'OpenTelemetry\Context\ContextKey',
] as $class) {
    echo $class, ': ', var_export((new ReflectionClass($class))->isFinal(), true), PHP_EOL;
}

echo "DONE\n";
?>
--EXPECT--
OpenTelemetry\API\Globals: true
OpenTelemetry\API\Trace\NonRecordingSpan: true
OpenTelemetry\API\Trace\Propagation\TraceContextPropagator: true
OpenTelemetry\API\Trace\SpanContext: true
OpenTelemetry\Context\Context: true
OpenTelemetry\Context\ContextKey: true
DONE
