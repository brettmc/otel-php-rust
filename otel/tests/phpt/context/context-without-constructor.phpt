--TEST--
Context methods report an error instead of crashing when the constructor was bypassed
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=1
--ENV--
OTEL_TRACES_EXPORTER=none
--FILE--
<?php
use OpenTelemetry\Context\Context;

// Context has a private constructor, but reflection reaches it anyway and leaves the
// internal state unpopulated. Every method has to report that rather than panicking,
// which would abort the whole PHP process.
$context = (new ReflectionClass(Context::class))->newInstanceWithoutConstructor();

foreach (['get', 'with', 'activate'] as $method) {
    try {
        $context->$method('key', 'value');
        echo "$method: no error\n";
    } catch (Throwable $e) {
        echo "$method: " . $e->getMessage() . "\n";
    }
}
echo "still running\n";
?>
--EXPECT--
get: Context was not created by a Context or Storage factory method
with: Context was not created by a Context or Storage factory method
activate: Context was not created by a Context or Storage factory method
still running
