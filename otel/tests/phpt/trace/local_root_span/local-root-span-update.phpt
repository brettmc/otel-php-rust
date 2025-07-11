--TEST--
Update local root span
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
use OpenTelemetry\API\Trace\LocalRootSpan;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$tracer = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url');

$root = $tracer->spanBuilder('root')->startSpan();
$scope = $root->activate();
$localRoot = LocalRootSpan::current();
$localRoot->updateName('updated');
$localRoot->setAttribute('key', 'value');
$root->end();
$scope->detach();

$span = Memory::getSpans()[0];
var_dump($span['name']);
var_dump($span['attributes']);
?>
--EXPECTF--
string(7) "updated"
array(1) {
  ["key"]=>
  string(5) "value"
}