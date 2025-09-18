--TEST--
Check if throwing an exception in post hook after IO operation will finish gracefully
--EXTENSIONS--
otel
--SKIPIF--
<?php echo 'skip does not segfault in rust'; ?>
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.cli.enabled=1
otel.log.level=warn
--FILE--
<?php
\OpenTelemetry\Instrumentation\hook(
    null,
    'helloWorld',
    fn() => var_dump('pre'),
    fn(object|string|null $scope, array $params, mixed $returnValue, ?Throwable $throwable) => throw new Exception('error'));


function helloWorld() {
  // below scandir call or any other
  // IO operation is necessary to trigger
  // segfault.
  scandir(".");
}

try {
helloWorld();
} catch(Exception) {}

--EXPECTF--
string(3) "pre"

Warning: helloWorld(): OpenTelemetry: post hook threw exception, class=null function=helloWorld message=error in %s