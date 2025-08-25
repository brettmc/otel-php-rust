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
otel.cli.enabled=1
--FILE--
<?php

?>
--EXPECTF--
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] %s message=RINIT::sapi module name is: cli
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] %s message=RINIT::tracing cli enabled by ini
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] %s message=RINIT::otel request is being traced, name=php:cli
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] %s message=RSHUTDOWN::auto-closing root span...
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] %s message=RSHUTDOWN::CONTEXT_STORAGE is empty :)
%A