--TEST--
Emit a log record from a logger
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_PROCESSOR=simple
OTEL_TRACES_EXPORTER=memory
OTEL_LOGS_PROCESSOR=simple
OTEL_LOGS_EXPORTER=memory
--INI--
otel.log.level=warn
otel.cli.enabled=1
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Logs\LogRecord;
use OpenTelemetry\API\Logs\MemoryLogsExporter;

$builder = Globals::tracerProvider()->getTracer("my_tracer")->spanBuilder('root');
$span = $builder->startSpan();
$scope = $span->activate();

$traceId = $span->getContext()->getTraceId();
$spanId = $span->getContext()->getSpanId();

$logger = Globals::loggerProvider()->getLogger("my_logger", '0.1', 'schema.url', ['one' => 1]);
$record = new LogRecord('test');
$record
    ->setSeverityNumber(9) //info
    ->setSeverityText('Info')
    ->setEventName('my_event')
    ->setTimestamp((int) (microtime(true) * 1e9))
    ->setAttributes([
        'a_bool' => true,
        'an_int' => 1,
        'a_float' => 1.1,
        'a_string' => 'foo',
    ])
    ->setAttribute('another_string', 'bar');
$logger->emit($record);
$span->end();
$scope->detach();

var_dump(MemoryLogsExporter::count());
$exported = MemoryLogsExporter::getLogs()[0];
var_dump($exported);
assert($exported['trace_id'] === $traceId);
assert($exported['span_id'] === $spanId);
?>
--EXPECTF--
int(1)
array(12) {
  ["body"]=>
  string(27) "Some(String(Owned("test")))"
  ["severity_number"]=>
  int(9)
  ["severity_text"]=>
  string(4) "Info"
  ["event_name"]=>
  string(8) "my_event"
  ["trace_id"]=>
  string(32) "%s"
  ["span_id"]=>
  string(16) "%s"
  ["trace_flags"]=>
  int(1)
  ["timestamp"]=>
  string(%d) "%d-%d-%dT%s"
  ["observed_timestamp"]=>
  string(%d) "%d-%d-%dT%s"
  ["attributes"]=>
  array(5) {
    ["a_bool"]=>
    string(13) "Boolean(true)"
    ["an_int"]=>
    string(6) "Int(1)"
    ["a_float"]=>
    string(11) "Double(1.1)"
    ["a_string"]=>
    string(20) "String(Owned("foo"))"
    ["another_string"]=>
    string(20) "String(Owned("bar"))"
  }
  ["instrumentation_scope"]=>
  array(4) {
    ["name"]=>
    string(9) "my_logger"
    ["version"]=>
    string(3) "0.1"
    ["schema_url"]=>
    string(10) "schema.url"
    ["attributes"]=>
    array(1) {
      ["one"]=>
      string(6) "I64(1)"
    }
  }
  ["resource"]=>
  array(8) {
    ["host.name"]=>
    string(%d) "String(Owned("%s"))"
    ["process.pid"]=>
    string(%d) "String(Owned("%d"))"
    ["process.runtime.name"]=>
    string(%d) "String(Owned("cli"))"
    ["process.runtime.version"]=>
    string(%d) "String(Owned("%d.%d.%d"))"
    ["service.name"]=>
    string(%d) "String(Static("unknown_service"))"
    ["telemetry.sdk.language"]=>
    string(%d) "String(Static("php"))"
    ["telemetry.sdk.name"]=>
    string(%d) "String(Static("ext-otel"))"
    ["telemetry.sdk.version"]=>
    string(%d) "String(Static("%d.%d.%d"))"
  }
}
