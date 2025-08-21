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
otel.cli.enabled=1
otel.log.level="warn"
--FILE--
<?php
use OpenTelemetry\API\Trace\SpanExporter\Memory;

eval('
class DemoClass {
    public function test(): void
    {
        var_dump("test");
    }
}
');

$demo = new DemoClass();
$demo->test();
$spans = Memory::getSpans();
$span = $spans[0];
assert($span['name'] === 'DemoClass::test');
var_dump($span['attributes']);
//var_dump(Memory::getSpans());
?>
--EXPECTF--
string(4) "test"
array(3) {
  ["code.function.name"]=>
  string(15) "DemoClass::test"
  ["code.file.path"]=>
  string(%d) "%s/autoinstrument-eval.php(4) : eval()'d code"
  ["code.line.number"]=>
  int(%d)
}