--TEST--
disable otel via OTEL_DISABLED in .env file
--EXTENSIONS--
otel
--INI--
otel.dotenv.per_request=On
otel.cli.enabled=On
otel.log.level=debug
--FILE--
<?php
?>
--EXPECTF--
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel: event src/lib.rs:%d message=OpenTelemetry::RINIT: OTEL_DISABLED is set to true, skipping initialization
%A