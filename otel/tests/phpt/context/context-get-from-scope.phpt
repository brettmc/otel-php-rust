--TEST--
Get context from scope
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
$scope = $context->activate();
var_dump($scope);

$ctx = $scope->context();
var_dump($ctx);
assert($ctx->get('some_key') === 'A');
assert(Context::getCurrent()->get('some_key') === 'A');
$scope->detach();
?>
--EXPECTF--
object(OpenTelemetry\Context\Scope)#2 (1) {
  ["context_id":"OpenTelemetry\Context\Scope":private]=>
  int(1)
}
object(OpenTelemetry\Context\Context)#3 (1) {
  ["context_id":"OpenTelemetry\Context\Context":private]=>
  int(1)
}