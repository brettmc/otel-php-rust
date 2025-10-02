--TEST--
Export a log: http/protobuf
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=On
otel.log.level="error"
otel.log.file="/dev/stdout"
--ENV--
OTEL_EXPORTER_OTLP_ENDPOINT=http://collector:4318
OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf
OTEL_EXPORTER_OTLP_TIMEOUT=1500
OTEL_SERVICE_NAME=test-http-protobuf
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Logs\LogRecord;

$logger = Globals::loggerProvider()->getLogger("my_logger", '0.1', 'schema.url', ['one' => 1]);
$record = new LogRecord('test');
$record->setSeverityNumber(9); //info
$logger->emit($record);

var_dump('done');
?>
--EXPECT--
string(4) "done"