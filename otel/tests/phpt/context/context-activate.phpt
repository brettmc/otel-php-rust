--TEST--
Activate context
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=none
--FILE--
<?php
use OpenTelemetry\Context\Context;

$context = Context::getCurrent();
$context = $context->with('some_key', 'A');
$scope = $context->activate();

assert(Context::getCurrent()->get('some_key') === 'A');
$scope->detach();
assert(Context::getCurrent()->get('some_key') === null);
?>
--EXPECT--
