--TEST--
Get current context
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=none
--FILE--
<?php
use OpenTelemetry\Context\Context;

$context = Context::getCurrent();
var_dump($context);
?>
--EXPECT--
object(OpenTelemetry\Context\Context)#1 (0) {
}