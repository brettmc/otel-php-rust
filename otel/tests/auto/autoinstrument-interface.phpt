--TEST--
Autoinstrument an interface
--EXTENSIONS--
otel
--SKIPIF--
<?php
if (PHP_VERSION_ID < 70200) {
    die("skip requires PHP 7.2+");
}
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.log.level="warn"
otel.log.file="/dev/stdout"
otel.cli.enabled=1
--FILE--
<?php
use OpenTelemetry\API\Trace\SpanExporter\Memory;

interface IDemo {
    public function foo(): void;
}
class DemoClass implements IDemo {
    public function foo(): void
    {
        var_dump("foo");
    }
}

$demo = new DemoClass();
$demo->foo();
assert(Memory::count() === 1);
$span = Memory::getSpans()[0];
assert($span['name'] === 'DemoClass::foo');
?>
--EXPECT--
string(3) "foo"