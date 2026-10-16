--TEST--
Context::with() returns a new Context and leaves the original unchanged
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=none
OTEL_LOGS_EXPORTER=none
OTEL_METRICS_EXPORTER=none
--FILE--
<?php
// `ContextInterface` documents an "immutable execution scoped propagation mechanism", and
// immutability is the half that is easy to break silently: if `with()` mutated the receiver
// and returned `$this`, deriving a context would change the one it was derived from, and
// every other holder of that original would see the change.
use OpenTelemetry\Context\Context;

$key = Context::createKey('k');
$original = Context::getCurrent();
$derived = $original->with($key, 'value');

// A distinct object, not the receiver.
var_dump($derived !== $original);
var_dump($derived instanceof Context);

// The original is still readable and does not carry the new value.
var_dump($original->get($key));
var_dump($derived->get($key));

// Deriving twice from the same context gives two independent children.
$a = $original->with($key, 'a');
$b = $original->with($key, 'b');
var_dump($a->get($key), $b->get($key), $original->get($key));

// Two keys with the SAME NAME are two different keys: lookup is by object identity, as
// upstream's `spl_object_id($key)` is, so a name collision must not be a key collision.
$first = Context::createKey('duplicate');
$second = Context::createKey('duplicate');
var_dump($first == $second);
$both = $original->with($first, 'first')->with($second, 'second');
var_dump($both->get($first), $both->get($second));
?>
--EXPECT--
bool(true)
bool(true)
NULL
string(5) "value"
string(1) "a"
string(1) "b"
NULL
bool(true)
string(5) "first"
string(6) "second"
