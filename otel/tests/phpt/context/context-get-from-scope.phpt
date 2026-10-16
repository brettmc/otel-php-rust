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

$key = Context::createKey('some_key');

$context = Context::getCurrent();
$context = $context->with($key, 'A');
$scope = $context->activate();
var_dump($scope);

$ctx = $scope->context();
var_dump($ctx);
// A Context reached back through its Scope must carry the same values as the one activated,
// which is only true because `activate()` copies them onto the new context id.
var_dump($ctx->get($key) === 'A');
var_dump(Context::getCurrent()->get($key) === 'A');
$scope->detach();
?>
--EXPECTF--
object(OpenTelemetry\Context\Scope)#%d (2) {
  ["context_id":"OpenTelemetry\Context\Scope":private]=>
  int(%d)
  ["localStorage":"OpenTelemetry\Context\Scope":private]=>
  NULL
}
object(OpenTelemetry\Context\Context)#%d (1) {
  ["context_id":"OpenTelemetry\Context\Context":private]=>
  int(%d)
}
bool(true)
bool(true)