--TEST--
Get logger from logger provider
--EXTENSIONS--
otel
--FILE--
<?php
use OpenTelemetry\API\Globals;

var_dump(Globals::loggerProvider()->getLogger("my_tracer", '0.1', 'schema.url'));
var_dump(Globals::loggerProvider()->getLogger("my_tracer", '0.1'));
var_dump(Globals::loggerProvider()->getLogger("my_tracer"));
var_dump(Globals::loggerProvider()->getLogger("my_tracer", '0.1', 'schema.url', ['one' => 1]));
?>
--EXPECTF--
object(OpenTelemetry\API\Logs\Logger)#%d (0) {
}
object(OpenTelemetry\API\Logs\Logger)#%d (0) {
}
object(OpenTelemetry\API\Logs\Logger)#%d (0) {
}
object(OpenTelemetry\API\Logs\Logger)#%d (0) {
}
