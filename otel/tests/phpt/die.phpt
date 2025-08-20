--TEST--
Check die
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.cli.enabled=1
otel.log.level="debug"
--FILE--
<?php
die('goodbye');
?>
--EXPECTF--
%Agoodbye%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::auto::plugin_manager: event src/auto/plugin_manager.rs:%d message=PluginManager::request_shutdown
%A