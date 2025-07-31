--TEST--
Autoinstrument a class + function is skipped if plugin is disabled
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
otel.auto.disabled_plugins="test,laminas, psr18"
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
var_dump(Memory::count());
?>
--EXPECT--
string(4) "test"
string(5) "inner"
int(0)