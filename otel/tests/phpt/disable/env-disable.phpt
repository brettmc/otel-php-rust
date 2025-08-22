--TEST--
disable otel via OTEL_SDK_DISABLED
--EXTENSIONS--
otel
--INI--
otel.log.level=debug
--ENV--
OTEL_SDK_DISABLED=true
--FILE--
<?php
?>
--EXPECTF--
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] %s message=OpenTelemetry::MINIT disabled%A