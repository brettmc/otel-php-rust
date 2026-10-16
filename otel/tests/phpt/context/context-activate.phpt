--TEST--
Activate context
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=none
OTEL_LOGS_EXPORTER=none
OTEL_METRICS_EXPORTER=none
--FILE--
<?php
use OpenTelemetry\Context\Context;

$key = Context::createKey('some_key');

$context = Context::getCurrent();
$context = $context->with($key, 'A');
$scope = $context->activate();

var_dump(Context::getCurrent()->get($key) === 'A');
$scope->detach();
var_dump(Context::getCurrent()->get($key) === null);
?>
--EXPECT--
bool(true)
bool(true)
