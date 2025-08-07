--TEST--
disable auto-instrumentation via .ini
--EXTENSIONS--
otel
--INI--
otel.log.level=debug
otel.cli.enabled=On
otel.auto.enabled=Off
--FILE--
<?php
?>
--EXPECTF--
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel: event src/lib.rs:%d message=OpenTelemetry::MINIT auto-instrumentation disabled
%A