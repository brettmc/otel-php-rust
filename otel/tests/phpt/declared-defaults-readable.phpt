--TEST--
Every declared default value can be read without crashing the process
--SKIPIF--
<?php
if (PHP_VERSION_ID < 80000) {
    // phper-sys only encodes default values into zend_internal_arg_info on PHP 8.0+; on 7.x
    // php_wrapper.c falls back to ZEND_ARG_TYPE_INFO, which has no default-value field, so
    // every default is dropped at registration. There is then no default-value AST for
    // reflection to read -- and so nothing that could be malformed enough to segfault, and
    // nothing for the "a zero count would make this test pass by checking nothing" guard
    // below to count.
    die("skip default values are not registered before PHP 8.0");
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
// `Argument::with_default_value` takes PHP *source*, which Zend compiles into a
// default-value AST. An invalid expression compiles to a malformed AST that segfaults the
// process the first time anything reads it -- and reflection is what reads it, so walking
// the extension with ReflectionClass is enough to trigger a SIGSEGV. Note that `""` is not
// a PHP expression; the empty string literal spells as `"''"`.
//
// A crash here is not a failed assertion but a *dead test*: no output at all. So the counts
// below are asserted, which turns a segfault into a visible diff rather than a silent pass.
$classes = array_merge(get_declared_classes(), get_declared_interfaces());
$read = 0;
$errors = [];

foreach ($classes as $class) {
    if (!str_starts_with($class, 'OpenTelemetry\\')) {
        continue;
    }
    $rc = new ReflectionClass($class);
    foreach ($rc->getMethods() as $method) {
        if ($method->getDeclaringClass()->getName() !== $class) {
            continue; // inherited: reported against the class that declares it
        }
        foreach ($method->getParameters() as $param) {
            if (!$param->isDefaultValueAvailable()) {
                continue;
            }
            // The read itself is the test.
            $default = $param->getDefaultValue();
            $read++;
            // A default must also satisfy the parameter's own type, or userland calling the
            // method at its short arity gets a value the signature says is impossible.
            $type = $param->getType();
            if ($type instanceof ReflectionNamedType && !$type->allowsNull() && $default === null) {
                $errors[] = sprintf('%s::%s(): $%s is %s but defaults to null',
                    $class, $method->getName(), $param->getName(), $type->getName());
            }
        }
    }
}

// A zero count would make this test pass by checking nothing.
var_dump($read > 0);
var_dump($errors);
?>
--EXPECT--
bool(true)
array(0) {
}
