--TEST--
Get a logger with instrumentation scope
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=1
--ENV--
OTEL_LOGS_EXPORTER=console
OTEL_LOGS_PROCESSOR=simple
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Logs\LogRecord;

$logger = Globals::loggerProvider()->getLogger('test', '1.0', 'https://schemas.opentelemetry.io/1.30.0', ['a_string' => 'foo', 'a_bool' => true, 'a_int' => 3]);
$record = new LogRecord();
$logger->emit($record);
//todo get record back from exporter and verify instrumentation scope
?>
--EXPECT--
array(4) {
  ["name"]=>
  string(4) "test"
  ["version"]=>
  string(3) "1.0"
  ["schema_url"]=>
  string(39) "https://schemas.opentelemetry.io/1.30.0"
  ["attributes"]=>
  array(3) {
    ["a_string"]=>
    string(3) "foo"
    ["a_bool"]=>
    bool(true)
    ["a_int"]=>
    int(3)
  }
}
