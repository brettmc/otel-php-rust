--TEST--
Check if post hook receives exception
--EXTENSIONS--
otel
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
    fn(object|null $obj, array $params, mixed $returnValue, ?Throwable $throwable) => var_dump($throwable?->getMessage()));

function helloWorld() {
    throw new Exception('error');
}

try {
    helloWorld();
} catch (Exception $e) {}
?>
--EXPECT--
string(5) "error"