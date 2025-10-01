--TEST--
Emit a log record from a logger
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=none
OTEL_LOGS_EXPORTER=console
--INI--
otel.log.level=warn
otel.cli.enabled=1
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Logs\LogRecord;

$logger = Globals::loggerProvider()->getLogger("my_logger", '0.1', 'schema.url', ['one' => 1]);
$record = new LogRecord('test');
$record->setSeverityNumber(9); //info
$logger->emit($record);
?>
--EXPECTF--
array(2){
  ["body"]=>
  string(4) "test"
  ["severity_text"]=>
  string(9) "INFO"
}
