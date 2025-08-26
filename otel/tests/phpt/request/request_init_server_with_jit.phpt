--TEST--
RINIT initializes $_SERVER with JIT enabled
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.log.level="debug"
otel.log.file="/dev/stdout"
otel.cli.enabled=1
auto_globals_jit=On
--FILE--
<?php
?>
--EXPECTF--
%A
%s message=JIT auto_globals_jit enabled, initializing $_SERVER
%A