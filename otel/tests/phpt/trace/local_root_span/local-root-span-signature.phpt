--TEST--
LocalRootSpan::current() declares a return type that exists
--SKIPIF--
<?php
if (PHP_VERSION_ID < 80000) {
    // phper-sys compiles ZEND_BEGIN_ARG_WITH_RETURN_OBJ_INFO_EX out below PHP 8.0, so
    // current() is registered with no return type and reflection reports "". The typo
    // this test guards against can therefore only be detected on 8.0+.
    die("skip class return types are not registered before PHP 8.0");
}
?>
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=1
--ENV--
OTEL_TRACES_EXPORTER=none
--FILE--
<?php
use OpenTelemetry\API\Trace\LocalRootSpan;

// The declared return type has to name a real interface: PHP resolves return types
// lazily, so a typo here is invisible at runtime but wrong in reflection output and
// unusable for any userland code type-checking against the signature.
$returnType = (string) (new ReflectionMethod(LocalRootSpan::class, 'current'))->getReturnType();
var_dump($returnType);
var_dump(interface_exists($returnType));

// Both branches of current() must satisfy it: a non-recording span when there is no
// local root, and a Span when there is one.
var_dump(LocalRootSpan::current() instanceof $returnType);
?>
--EXPECT--
string(37) "OpenTelemetry\API\Trace\SpanInterface"
bool(true)
bool(true)
