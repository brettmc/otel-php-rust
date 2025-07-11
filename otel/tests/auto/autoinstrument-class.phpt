--TEST--
Autoinstrument a class + function
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
otel.cli.enable=1
otel.log.level="warn"
--FILE--
<?php
use OpenTelemetry\API\Trace\SpanExporter\Memory;

class DemoClass {
    public function test(): void
    {
        var_dump("test");
        $this->inner();
    }
    private function inner(): void
    {
        var_dump("inner");
    }
}

$demo = new DemoClass();
$demo->test();
$spans = Memory::getSpans();
$test = $spans[0];
$inner = $spans[1];
assert($test['name'] === 'DemoClass::inner');
assert($inner['name'] === 'DemoClass::test');
assert($test['span_context']['trace_id'] === $inner['span_context']['trace_id']);
assert($test['parent_span_id'] === $inner['span_context']['span_id']);
//var_dump(Memory::getSpans());
?>
--EXPECT--
string(4) "test"
string(5) "inner"