--TEST--
ContextKey's name is a readonly ?string, as upstream's promoted property is
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=1
--ENV--
OTEL_TRACES_EXPORTER=none
--FILE--
<?php
use OpenTelemetry\Context\ContextKey;

// Readonly needs 8.1 and typed properties 8.0. Below that the property is a plain one, so
// the expectations differ by version but every version must reach DONE. No `void`, which
// is 7.1 syntax.
function check(string $label, $actual, $expected)
{
    echo $label, ': ', $actual === $expected
        ? 'ok'
        : sprintf('got %s, expected %s', var_export($actual, true), var_export($expected, true)), PHP_EOL;
}

$property = new ReflectionProperty(ContextKey::class, 'name');
check('type', PHP_VERSION_ID >= 80000 ? (string) $property->getType() : null, PHP_VERSION_ID >= 80000 ? '?string' : null);
check('readonly', PHP_VERSION_ID >= 80100 ? $property->isReadOnly() : null, PHP_VERSION_ID >= 80100 ? true : null);

// A second constructor call must not rename the key. The Error comes from the engine's
// write inside the handler, so it has to surface as an exception, not a panic.
$key = new ContextKey('original');
try {
    $key->__construct('other');
    $outcome = 'renamed';
} catch (Error $e) {
    $outcome = $e->getMessage();
}
check('reconstruct', $outcome, PHP_VERSION_ID >= 80100
    ? 'Cannot modify readonly property OpenTelemetry\Context\ContextKey::$name'
    : 'renamed');
check('name', $key->name(), PHP_VERSION_ID >= 80100 ? 'original' : 'other');

// unserialize() skips the constructor, leaving a typed property uninitialized. name() must
// not hand that UNDEF slot to userland. phper reads properties silently, and a silent read
// of an uninitialized property gives null, where upstream throws "must not be accessed
// before initialization". Null is what `?string` allows.
$bare = unserialize(sprintf('O:%d:"%s":0:{}', strlen(ContextKey::class), ContextKey::class));
try {
    $outcome = $bare->name();
} catch (Error $e) {
    $outcome = $e->getMessage();
}
check('uninitialized', $outcome, null);

echo "DONE\n";
?>
--EXPECT--
type: ok
readonly: ok
reconstruct: ok
name: ok
uninitialized: ok
DONE
