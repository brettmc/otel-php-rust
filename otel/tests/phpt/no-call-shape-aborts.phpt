--TEST--
Calling every method with no arguments raises a PHP error rather than aborting the process
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=1
--ENV--
OTEL_TRACES_EXPORTER=none
OTEL_LOGS_EXPORTER=none
--FILE--
<?php
// phper derives `required_num_args` from the number of arguments a method *declares*, so a
// method that declares none accepts a zero-argument call -- and the handler then indexes
// `arguments[0]`, which panics. A Rust panic cannot unwind across `extern "C"`, so it
// aborts the process (SIGABRT): an fpm or Apache worker dies mid-request, and userland
// cannot catch it.
//
// Rather than pin the arity of each method individually -- which says nothing about the
// next method added without a declaration -- call *everything* with no arguments and
// require that the process survive. An ArgumentCountError or TypeError is the correct
// outcome; what is being asserted is that something catchable happens at all.
//
// A regression aborts before reaching the final echo, so the missing output fails the
// test. Note that nothing is printed per method: an abort kills the process, so partial
// output could not be trusted to show how far the sweep got anyway.

$swept = 0;
$survived = [];

foreach (get_declared_classes() as $class) {
    // strpos rather than str_starts_with: this test runs on PHP 7, where that function
    // does not exist. An abort is worth catching on every version we support.
    if (strpos($class, 'OpenTelemetry\\') !== 0) {
        continue;
    }
    $rc = new ReflectionClass($class);
    foreach ($rc->getMethods() as $method) {
        $name = $method->getName();
        if ($method->getDeclaringClass()->getName() !== $class) {
            continue; // inherited: swept against the class that declares it
        }
        if ($name === '__construct' || $name === '__destruct') {
            continue; // not callable this way, and __destruct runs on teardown regardless
        }
        $swept++;
        try {
            if ($method->isStatic()) {
                $class::$name();
            } else {
                // Most of these classes have a private constructor and are handed out by a
                // factory. An instance with no state is exactly the awkward case: the
                // handler must report it, not panic on an empty Option.
                $rc->newInstanceWithoutConstructor()->$name();
            }
        } catch (Throwable $e) {
            // Catchable is the whole point.
            $survived[] = $name;
        }
    }
}

// A zero count would make this test pass by sweeping nothing.
var_dump($swept > 0);
// At least some of these genuinely require arguments, so an empty list would mean the
// calls were not actually reaching the handlers.
var_dump($survived !== []);
echo "process survived\n";
?>
--EXPECT--
bool(true)
bool(true)
process survived
