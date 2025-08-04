--TEST--
Local root span does not exist
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
--INI--
otel.log.level="error"
otel.log.file="/dev/stdout"
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\LocalRootSpan;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$tracer = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url');

assert(LocalRootSpan::current()->getContext()->isValid() === false);

$localRoot = LocalRootSpan::current();
$localRoot->updateName('no-op');
$localRoot->end();
assert(Memory::count() === 0);
?>
--EXPECTF--
