--TEST--
disable otel via OTEL_SDK_DISABLED in .env file is more powerful than env
--EXTENSIONS--
otel
--INI--
otel.env.dotenv.enabled=On
otel.cli.enabled=On
otel.log.level=debug
--ENV--
OTEL_SDK_DISABLED=false
--FILE--
<?php
?>
--EXPECTF--
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] %s message=OpenTelemetry::RINIT: SDK disabled, skipping initialization
%A