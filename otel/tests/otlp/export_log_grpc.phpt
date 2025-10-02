--TEST--
Export a log: grpc
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=On
otel.log.level="error"
otel.log.file="/dev/stdout"
--ENV--
OTEL_EXPORTER_OTLP_ENDPOINT=http://collector:4317
OTEL_EXPORTER_OTLP_PROTOCOL=grpc
OTEL_EXPORTER_OTLP_TIMEOUT=1500
OTEL_SERVICE_NAME=test-grpc
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Logs\LogRecord;

$logger = Globals::loggerProvider()->getLogger("my_logger", '0.1', 'schema.url', ['one' => 1]);
$record = new LogRecord('test');
$record
    ->setSeverityNumber(9) //info
    ->setSeverityText('Info')
    ->setEventName('my_event')
    ->setTimestamp((int)(microtime(true) * 1e9))
    ->setAttributes([
        'a_bool' => true,
        'an_int' => 1,
        'a_float' => 1.1,
        'a_string' => 'foo',
        'string_array' => ['one', 'two', 'three'],
        'int_array' => [1, 2, 3],
        'float_array' => [1.1, 2.2, 3.3],
        'bool_array' => [true, false, true],
    ])
    ->setAttribute('another_string', 'bar');
$logger->emit($record);

var_dump('done');
?>
--EXPECT--
string(4) "done"