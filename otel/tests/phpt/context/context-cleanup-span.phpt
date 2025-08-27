--TEST--
Internal context storage empty after use
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
--INI--
otel.log.level=debug
otel.log.file="/dev/stdout"
otel.cli.enabled=1
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\Span;
use OpenTelemetry\Context\Context;

$tracer = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url');

// "pre hook"
var_dump('pre: start span');
$span = $tracer->spanBuilder('root')->startSpan();
var_dump('post: start span');
var_dump('pre: get current context');
$context = Context::getCurrent();
var_dump('post: get current context');
var_dump('pre: storage attach');
Context::storage()->attach($span->storeInContext($context));
var_dump('post: storage attach');
unset($span);

// "post hook"
var_dump('pre: get scope from storage');
$scope = Context::storage()->scope();
var_dump('post: get scope from storage');
var_dump('pre: detach scope');
$scope->detach();
var_dump('post: detach scope');
var_dump('pre: get span from scope context');
$span = Span::fromContext($scope->context());
var_dump('post: get span from scope context');
var_dump('pre: unset scope');
unset($scope);
var_dump('post: unset scope');
var_dump('pre: span end');
$span->end();
var_dump('post: span end');

?>
--EXPECTF--
%A
string(15) "pre: start span"
%s message=SpanBuilder::No parent context, using Context::current()
%s message=SpanBuilder::Starting span
string(16) "post: start span"
string(24) "pre: get current context"
string(25) "post: get current context"
string(19) "pre: storage attach"
%s message=Storing context instance 1 (ref count after clone = 2)
%s message=Attaching context instance 1
%s message=Getting context instance 1
%s message=Cloned context instance 1 (ref count after clone = 3)
%s message=Before attach: context instance 1 has ref count = 4
%s message=Context::__destruct for context_id = Some(1)
%s message=Maybe remove context for instance 1
%s message=Cannot remove context instance 1 (ref count = 2, still in use)
string(20) "post: storage attach"
string(27) "pre: get scope from storage"
string(28) "post: get scope from storage"
string(17) "pre: detach scope"
%s message=Detaching context instance 1
string(18) "post: detach scope"
string(32) "pre: get span from scope context"
%s message=Getting context instance 1
%s message=Cloned context instance 1 (ref count after clone = 2)
%s message=Context::__destruct for context_id = Some(1)
%s message=Maybe remove context for instance 1
%s message=Cannot remove context instance 1 (ref count = 2, still in use)
string(33) "post: get span from scope context"
string(16) "pre: unset scope"
string(17) "post: unset scope"
string(13) "pre: span end"
%s message=Getting context instance 1
%s message=Cloned context instance 1 (ref count after clone = 2)
%s message=Span::Ending Span (SpanRef)
%s message=Maybe remove context for instance 1
%s message=Removing context instance 1 (ref count = 1, no external holders)
string(14) "post: span end"
%s message=Context::__destruct for context_id = None
%A
%s message=RSHUTDOWN::CONTEXT_STORAGE is empty :)%A