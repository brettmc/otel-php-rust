--TEST--
Set and retrieve a value from context
--SKIPIF--
<?php die('skip Keys not handled correctly, only stores string, cannot store spans');
?>
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
$context = $context->with('some_key', 'A');
var_dump($context->get('some_key'));
$context = $context->with('another_key', 'B');
var_dump($context->get('another_key'));
var_dump($context->get('some_key'));
?>
--EXPECT--
string(1) "A"
string(1) "B"
string(1) "A"