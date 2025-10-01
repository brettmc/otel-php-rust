--TEST--
Check if post hook receives exception
--EXTENSIONS--
otel
--SKIPIF--
<?php if (version_compare(PHP_VERSION, '8.0.0', '>=')) echo 'skip requires php 7.x'; ?>
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
    null,
    function($obj, array $params, $returnValue, Throwable $throwable) { var_dump($throwable->getMessage());}
);

function helloWorld() {
    throw new Exception('error');
}

try {
    helloWorld();
} catch (Exception $e) {
    var_dump($e);
}
?>
--EXPECTF--
string(5) "error"
object(Exception)%A