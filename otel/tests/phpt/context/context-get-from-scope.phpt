--TEST--
Get context from scope
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

$ctx = $scope->context();
assert(Context::current() === $ctx);
assert($ctx->get('some_key') === 'A');
assert(Context::current()->get('some_key') === 'A');
$scope->detach();

?>
--EXPECT--
object(OpenTelemetry\Context\Context)#1 (0) {
}