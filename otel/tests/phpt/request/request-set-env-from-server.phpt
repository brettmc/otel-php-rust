--TEST--
Set OTEL_* environment from $_SERVER, restore env on RSHUTDOWN
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SDK_DISABLED=true
OTEL_SERVICE_NAME=php-test
MY_FOO=bar
--INI--
otel.log.level=debug
otel.log.file="/dev/stdout"
otel.env.set_from_server=On
otel.cli.enabled=On
--FILE--
<?php
var_dump(getenv('OTEL_TRACES_EXPORTER'));
var_dump(getenv('OTEL_SDK_DISABLED'));
var_dump(getenv('OTEL_SERVICE_NAME'));
var_dump(getenv('MY_FOO'));
/*

*/
?>
--EXPECTF--
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:%d message=Backing up OTEL_* environment variables%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:%d message=Set environment variable from $_SERVER: OTEL_TRACES_EXPORTER=memory
%A
string(6) "memory"
string(4) "true"
string(8) "php-test"
string(3) "bar"
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:%d message=Restoring environment variables from backup%A
%A