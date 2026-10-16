--TEST--
A Scope is an ArrayAccess scratch map, per ContextStorageScopeInterface
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=1
--ENV--
OTEL_TRACES_EXPORTER=none
OTEL_LOGS_EXPORTER=none
OTEL_METRICS_EXPORTER=none
--FILE--
<?php
// `ContextStorageScopeInterface extends ScopeInterface, ArrayAccess` upstream, where the
// array half is per-scope scratch space instrumentation can use without maintaining its own
// map keyed by scope (upstream's ContextStorageNode calls it `$localStorage`). Nothing in
// this extension reads these offsets; they exist for callers.
use OpenTelemetry\Context\Context;
use OpenTelemetry\Context\ContextStorageScopeInterface;

$scope = Context::getCurrent()->activate();
var_dump($scope instanceof ContextStorageScopeInterface);
var_dump($scope instanceof ArrayAccess);

// Empty until written, and an absent offset reads as null rather than raising.
var_dump(isset($scope['a']));
var_dump($scope['a']);

$scope['a'] = 'first';
$scope['b'] = ['nested' => true];
var_dump(isset($scope['a']), $scope['a']);
var_dump($scope['b']);

// Overwriting replaces, and unsetting removes only what was named.
$scope['a'] = 'second';
var_dump($scope['a']);
unset($scope['a']);
var_dump(isset($scope['a']), isset($scope['b']));

// Two scopes have separate maps -- the point of per-scope storage.
$inner = Context::getCurrent()->activate();
var_dump(isset($inner['b']));
$inner['b'] = 'own';
var_dump($inner['b'], $scope['b']);
$inner->detach();

var_dump($scope->detach());
// Still readable after detaching: the Scope object outlives its activation, and a caller
// holding it may still want its bookkeeping.
var_dump($scope['b']);
?>
--EXPECT--
bool(true)
bool(true)
bool(false)
NULL
bool(true)
string(5) "first"
array(1) {
  ["nested"]=>
  bool(true)
}
string(6) "second"
bool(false)
bool(true)
bool(false)
string(3) "own"
array(1) {
  ["nested"]=>
  bool(true)
}
int(0)
array(1) {
  ["nested"]=>
  bool(true)
}
