--TEST--
A userland span declaring upstream's setAttribute signature satisfies SpanInterface
--SKIPIF--
<?php
if (PHP_VERSION_ID < 80000) {
    // Union types are PHP 8.0 syntax, so the --FILE-- below is a parse error on 7.x.
    die("skip union types require PHP 8.0");
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
use OpenTelemetry\API\Trace\SpanInterface;

// Parameters are contravariant, so an interface declaring a wider parameter than upstream
// rejects an implementation copied from upstream. With `mixed` on the interface this class
// was a compile-time fatal, and the test died before printing anything.
abstract class UpstreamShapedSpan implements SpanInterface
{
    public function setAttribute(string $key, bool|int|float|string|array|null $value): SpanInterface
    {
        return $this;
    }
}

$type = (new ReflectionMethod(SpanInterface::class, 'setAttribute'))->getParameters()[1]->getType();
echo $type, PHP_EOL;

// The concrete classes stay `mixed`, a legal widening that upstream's untyped classes share,
// and must still match each other exactly.
foreach (['OpenTelemetry\API\Trace\Span', 'OpenTelemetry\API\Trace\NonRecordingSpan'] as $class) {
    echo $class, ': ', (new ReflectionMethod($class, 'setAttribute'))->getParameters()[1]->getType(), PHP_EOL;
}

echo "DONE\n";
?>
--EXPECT--
array|string|int|float|bool|null
OpenTelemetry\API\Trace\Span: mixed
OpenTelemetry\API\Trace\NonRecordingSpan: mixed
DONE
