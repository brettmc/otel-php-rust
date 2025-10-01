--TEST--
Test fetching providers from Globals
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_LOGS_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.cli.enabled=1
--FILE--
<?php
use OpenTelemetry\API\Globals;

$tracerProvider = Globals::tracerProvider();
var_dump(get_class($tracerProvider));
$loggerProvider = Globals::loggerProvider();
var_dump(get_class($loggerProvider));
?>
--EXPECT--
string(38) "OpenTelemetry\API\Trace\TracerProvider"
string(37) "OpenTelemetry\API\Logs\LoggerProvider"