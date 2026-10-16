--TEST--
Storage::attach(Context::getCurrent()) works, and current() is an instance method
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
// `$storage->attach(Context::getCurrent())` is the first thing any caller of the storage API
// does, and it has to work: `Context::getCurrent()` reports a context_id of 0 whenever
// nothing is attached yet, meaning "no stored instance", which is the ordinary starting
// state and not an error.
//
// `current()` is asserted to be an *instance* method, as upstream declares it on
// ContextStorageInterface. A static method cannot satisfy an interface's instance method, so
// declaring it either way is the difference between the interface being honest and not.
use OpenTelemetry\Context\Context;
use OpenTelemetry\Context\ContextInterface;
use OpenTelemetry\Context\ContextStorageInterface;
use OpenTelemetry\Context\ContextStorageScopeInterface;

$storage = Context::storage();
var_dump($storage instanceof ContextStorageInterface);

$method = new ReflectionMethod(ContextStorageInterface::class, 'current');
var_dump($method->isStatic());

// Nothing attached yet.
var_dump($storage->scope());
var_dump($storage->current() instanceof ContextInterface);

$scope = $storage->attach(Context::getCurrent());
var_dump($scope instanceof ContextStorageScopeInterface);
var_dump($storage->scope() !== null);
var_dump($scope->detach());
var_dump($storage->scope());

// current() must report the context that is actually attached, including its PHP-side
// values -- so it has to consult the guard stack and carry the attached context's id. A
// fresh context with no id would answer null here while looking otherwise correct.
$key = Context::createKey('k');
$scope = $storage->attach(Context::getCurrent()->with($key, 'visible'));
var_dump($storage->current()->get($key));
$scope->detach();
var_dump($storage->current()->get($key));
?>
--EXPECT--
bool(true)
bool(false)
NULL
bool(true)
bool(true)
bool(true)
int(0)
NULL
string(7) "visible"
NULL
