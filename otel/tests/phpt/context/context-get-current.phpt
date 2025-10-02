--TEST--
Get current context
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=none
OTEL_LOGS_EXPORTER=none
OTEL_METRICS_EXPORTER=none
--FILE--
<?php
use OpenTelemetry\Context\Context;

$context = Context::getCurrent();
var_dump($context);
?>
--EXPECT--
object(OpenTelemetry\Context\Context)#1 (1) {
  ["context_id":"OpenTelemetry\Context\Context":private]=>
  int(0)
}