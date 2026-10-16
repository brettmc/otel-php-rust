--TEST--
Optional arguments with a known default can be passed by name
--SKIPIF--
<?php
if (PHP_VERSION_ID < 80000) {
    // Named arguments are PHP 8.0 syntax, so the --FILE-- below is a parse error on 7.x
    // rather than a failed assertion. There is nothing to assert there either: phper-sys
    // only encodes default values into zend_internal_arg_info on 8.0+ (php_wrapper.c falls
    // back to ZEND_ARG_TYPE_INFO, which has no default-value field), so no argument has a
    // known default to skip past in the first place.
    die("skip named arguments require PHP 8.0");
}
?>
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=1
--ENV--
OTEL_TRACES_EXPORTER=none
OTEL_LOGS_EXPORTER=none
--FILE--
<?php
use OpenTelemetry\API\Globals;

// Passing an optional argument by name requires the declaration to record a real default.
// `.optional()` only clears the *required* flag, and PHP refuses to skip a parameter whose
// default it cannot see:
//   ArgumentCountError: Argument #2 ($version) must be passed explicitly,
//                       because the default value is not known
// `.with_default_value("null")` records one, which is also what leaves the handler a slot to
// read instead of an absent one.
//
// Not every argument can be fixed this way: phper routes a class-typed argument through
// `ZEND_ARG_OBJ_INFO`, which has no default-value field, so the default is dropped and the
// named call still fails. That is why the arguments asserted here are the builtin-typed ones.

$tracerProvider = Globals::tracerProvider();
var_dump($tracerProvider->getTracer('n', version: 'v') !== null);
var_dump($tracerProvider->getTracer('n', schemaUrl: 's') !== null);
var_dump($tracerProvider->getTracer('n', attributes: ['a' => 'b']) !== null);

$loggerProvider = Globals::loggerProvider();
var_dump($loggerProvider->getLogger('n', version: 'v') !== null);
var_dump($loggerProvider->getLogger('n', schemaUrl: 's') !== null);
var_dump($loggerProvider->getLogger('n', attributes: ['a' => 'b']) !== null);
?>
--EXPECT--
bool(true)
bool(true)
bool(true)
bool(true)
bool(true)
bool(true)
