--TEST--
dotenv modified vars are restored on request shutdown
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=On
otel.dotenv.per_request=On
otel.log.level=debug
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
OTEL_SERVICE_NAME=to-restore
OTEL_RESOURCE_ATTRIBUTES=service.namespace=to-restore
OTEL_SDK_DISABLED=to-restore
--FILE--
<?php
//do nothing
?>
--EXPECTF--
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:%d message=Backing up environment variables: %s
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:%d message=Set environment variable OTEL_%s=%Sfrom-dotenv
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:%d message=Set environment variable OTEL_%s=%Sfrom-dotenv
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:%d message=Set environment variable OTEL_%s=%Sfrom-dotenv
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:%d message=Restoring environment variables from backup
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:%d message=Restoring environment variable OTEL_%s=%Sto-restore
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:%d message=Restoring environment variable OTEL_%s=%Sto-restore
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:%d message=Restoring environment variable OTEL_%s=%Sto-restore
%A