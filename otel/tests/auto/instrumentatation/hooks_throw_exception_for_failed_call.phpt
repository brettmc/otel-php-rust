--TEST--
Check if exceptions thrown in hooks interfere with internal exceptions
--EXTENSIONS--
otel
--SKIPIF--
<?php
if (version_compare(PHP_VERSION, '8.0.0', '<')) {
    die('skip required php 8.x');
}
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.cli.enabled=1
otel.log.level=warn
--FILE--
<?php
function helloWorld($argument) {
    var_dump('inside');
}
\OpenTelemetry\Instrumentation\hook(
    null,
    'helloWorld',
    pre: static function () : void {
        throw new \Exception('pre');
    },
    post: static function () : void {
        throw new \Exception('post');
    }
);
helloWorld();
?>
--EXPECTF--

%sOpenTelemetry: pre hook threw exception, class=null function=helloWorld message=pre in %s
%sOpenTelemetry: post hook threw exception, class=null function=helloWorld message=post in %s

Fatal error: Uncaught ArgumentCountError: Too few arguments to function helloWorld(), 0 passed in %s
Stack trace:
#0 %s: helloWorld()
#1 {main}
  thrown in %s
