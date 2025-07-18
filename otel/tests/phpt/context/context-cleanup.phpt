--TEST--
Activate context
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=none
--INI--
otel.log.level="trace"
otel.log.file="/dev/stdout"
otel.cli.enable=1
--FILE--
<?php
use OpenTelemetry\Context\Context;

$context = Context::getCurrent();
var_dump("activate context");
$scope = $context->activate();
//context is now stored
var_dump("unsetting context");
unset($context);
//context should not have been removed from storage
var_dump("detaching scope");
$scope->detach();
var_dump("scope detached");
?>
--EXPECTF--
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:%d message=RINIT::not auto-creating root span...
string(16) "activate context"
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::context::storage: event src/context/storage.rs:%d message=Storing context instance 1 (ref count after clone = 3)
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::context::context: event src/context/context.rs:%d message=Storing context: 1
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::context::storage: event src/context/storage.rs:%d message=Attaching context instance 1
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::context::storage: event src/context/storage.rs:%d message=Getting context instance 1
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::context::storage: event src/context/storage.rs:%d message=Cloned context instance 1 (ref count after clone = 4)
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::context::storage: event src/context/storage.rs:%d message=Before attach: context instance 1 has ref count = 5
string(17) "unsetting context"
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::context::context: event src/context/context.rs:%d message=Context::__destruct for context_id = 1
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::context::storage: event src/context/storage.rs:%d message=Maybe remove context for instance 1
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::context::storage: event src/context/storage.rs:%d message=Cannot remove context instance 1 (ref count = 2, still in use)
string(15) "detaching scope"
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::context::storage: event src/context/storage.rs:%d message=Detaching context instance 1
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::context::storage: event src/context/storage.rs:%d message=Maybe remove context for instance 1
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::context::storage: event src/context/storage.rs:%d message=Removing context instance 1 (ref count = 1, no external holders)
string(14) "scope detached"
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel: event src/lib.rs:%d message=OpenTelemetry::RSHUTDOWN
%A