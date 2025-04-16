--TEST--
Force auto root span for CLI, check context is empty when finished
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=none
--INI--
otel.log.level="trace"
otel.log.file="/dev/stdout"
otel.cli.create_root_span="On"
--FILE--
<?php

?>
--EXPECTF--
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] OpenTelemetry::MINIT
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] OpenTelemetry::RSHUTDOWN
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:%d message=RSHUTDOWN::auto-closing root span...
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:%d message=RSHUTDOWN::CONTEXT_STORAGE is empty :)
%A