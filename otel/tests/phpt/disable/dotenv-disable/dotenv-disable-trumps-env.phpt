--TEST--
disable otel via OTEL_DISABLED in .env file is more powerful than env
--EXTENSIONS--
otel
--INI--
otel.dotenv.per_request=On
otel.log.level=debug
--ENV--
OTEL_DISABLED=false
--FILE--
<?php
?>
--EXPECTF--
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel: event src/lib.rs:%d message=OpenTelemetry::MINIT disabled