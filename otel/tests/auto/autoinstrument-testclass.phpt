--TEST--
Autoinstrument a method with discarded return value
--EXTENSIONS--
otel
--SKIPIF--
<?php
if (PHP_VERSION_ID < 80000) {
    die("skip requires PHP 8.0+");
}
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.cli.enabled=1
otel.log.level="debug"
--FILE--
<?php
namespace OpenTelemetry\Test;

use OpenTelemetry\API\Trace\SpanExporter\Memory;

interface ITestClass {
    public function getString(): string;
    public function throwException(): void;
}

class TestClass implements ITestClass {
    public function getString(): string {
        return "Hello, World!";
    }
    public function throwException(): void {
        throw new \Exception("This is a test exception");
    }
}

$c = new TestClass();
//not storing the result of getString in php7 leads to the return value being optimized out
$c->getString();
try {
    $c->throwException();
} catch (\Exception $e) {
    // do nothing
}
?>
--EXPECTF--
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::auto::plugins::test: event src/auto/plugins/test.rs:%d message=TestClassHandler: pre_callback called
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::auto::plugins::test: event src/auto/plugins/test.rs:%d message=TestClassHandler: post_callback called
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::auto::plugins::test: event src/auto/plugins/test.rs:%d message=retval type: TypeInfo { base_name: "string", base: 6, raw: 6 }
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::auto::plugins::test: event src/auto/plugins/test.rs:%d message=exception: None
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::auto::plugins::test: event src/auto/plugins/test.rs:%d message=TestClassHandler: pre_callback called
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::auto::plugins::test: event src/auto/plugins/test.rs:%d message=TestClassHandler: post_callback called
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::auto::plugins::test: event src/auto/plugins/test.rs:%d message=retval type: TypeInfo { base_name: "null", base: 1, raw: 1 }
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::auto::plugins::test: event src/auto/plugins/test.rs:%d message=exception: Some(ZObj { class: "Exception", handle: 2 })
%A