--TEST--
disable otel via OTEL_DISABLED
--EXTENSIONS--
otel
--INI--
otel.log.level=debug
--ENV--
OTEL_DISABLED=true
--FILE--
<?php
?>
--EXPECTF--
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel: event src/lib.rs:%d message=OpenTelemetry::MINIT disabled%A