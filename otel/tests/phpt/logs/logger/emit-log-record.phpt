--TEST--
Emit a log record from a logger
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=none
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

$logger = Globals::loggerProvider()->getLogger("my_logger", '0.1', 'schema.url', ['one' => 1]);
$record = new LogRecord('test');
$record->setSeverityNumber(9); //info
$logger->emit($record);
var_dump(MemoryLogsExporter::count());
$exported = MemoryLogsExporter::getLogs()[0];
var_dump($exported);
?>
--EXPECTF--
int(1)
array(8) {
  ["body"]=>
  string(27) "Some(String(Owned("test")))"
  ["severity_number"]=>
  int(9)
  ["severity_text"]=>
  string(0) ""
  ["timestamp"]=>
  int(0)
  ["observed_timestamp"]=>
  int(%d)
  ["attributes"]=>
  array(0) {
  }
  ["instrumentation_scope"]=>
  array(3) {
    ["name"]=>
    string(9) "my_logger"
    ["version"]=>
    string(3) "0.1"
    ["schema_url"]=>
    string(10) "schema.url"
  }
  ["resource"]=>
  array(8) {
    ["host.name"]=>
    string(29) "String(Owned("%s"))"
    ["process.pid"]=>
    string(22) "String(Owned("%d"))"
    ["process.runtime.name"]=>
    string(20) "String(Owned("cli"))"
    ["process.runtime.version"]=>
    string(23) "String(Owned("%d.%d.%d"))"
    ["service.name"]=>
    string(33) "String(Static("unknown_service"))"
    ["telemetry.sdk.language"]=>
    string(21) "String(Static("php"))"
    ["telemetry.sdk.name"]=>
    string(26) "String(Static("ext-otel"))"
    ["telemetry.sdk.version"]=>
    string(24) "String(Static("%d.%d.%d"))"
  }
}
