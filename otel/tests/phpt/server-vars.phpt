--TEST--
Get server vars
--EXTENSIONS--
otel
--FILE--
<?php
\OpenTelemetry\API\Globals::getServerVars();
?>
--EXPECT--
bool(true)