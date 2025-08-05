--TEST--
zf1 route hook adds extra data to root span
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.log.level=warn
otel.cli.enabled=On
otel.cli.create_root_span=On
--FILE--
<?php
use OpenTelemetry\API\Trace\SpanExporter\Memory;
use OpenTelemetry\API\Trace\LocalRootSpan;

include __DIR__ . '/public/index.php';

$localRoot = LocalRootSpan::current();
$localRoot->end();
var_dump(Memory::count());
$spans = Memory::getSpans();
$one = $spans[0];
$attributes = $one['attributes'];
var_dump($attributes);
?>
--EXPECTF--
%A
array(6) {
  ["url.full"]=>
  string(0) ""
  ["http.request.method"]=>
  string(0) ""
  ["php.framework.name"]=>
  string(3) "zf1"
  ["php.framework.module.name"]=>
  string(7) "default"
  ["php.framework.controller.name"]=>
  string(5) "index"
  ["php.framework.action.name"]=>
  string(5) "index"
}
%A