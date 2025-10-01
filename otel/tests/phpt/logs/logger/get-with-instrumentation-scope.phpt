--TEST--
Get a logger with instrumentation scope
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=1
--ENV--
OTEL_TRACES_EXPORTER=none
OTEL_LOGS_PROCESSOR=simple
OTEL_LOGS_EXPORTER=memory
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Logs\LogRecord;
use OpenTelemetry\API\Logs\MemoryLogsExporter;

$logger = Globals::loggerProvider()->getLogger('test', '1.0', 'https://schemas.opentelemetry.io/1.30.0', ['a_string' => 'foo', 'a_bool' => true, 'a_int' => 3]);
$record = new LogRecord();
$logger->emit($record);
$exported = MemoryLogsExporter::getLogs()[0];
$scope = $exported['instrumentation_scope'];
var_dump($scope);
?>
--EXPECTF--
array(4) {
  ["name"]=>
  string(4) "test"
  ["version"]=>
  string(3) "1.0"
  ["schema_url"]=>
  string(%d) "https://schemas.opentelemetry.io/1.%d.%d"
  ["attributes"]=>
  array(3) {
    ["a_string"]=>
    string(20) "String(Owned("foo"))"
    ["a_bool"]=>
    string(10) "Bool(true)"
    ["a_int"]=>
    string(6) "I64(3)"
  }
}
