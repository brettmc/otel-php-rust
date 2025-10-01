--TEST--
Check if multiple hooks are invoked
--EXTENSIONS--
otel
--SKIPIF--
<?php if (PHP_VERSION_ID < 80000) echo 'skip requires php8'; ?>
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.cli.enabled=1
otel.log.level=warn
--FILE--
<?php
\OpenTelemetry\Instrumentation\hook(null, 'helloWorld', fn() => var_dump('PRE_1'), fn() => var_dump('POST_1'));
\OpenTelemetry\Instrumentation\hook(null, 'helloWorld', fn() => var_dump('PRE_2'), fn() => var_dump('POST_2'));

function helloWorld() {
    var_dump('CALL');
}

helloWorld();
?>
--EXPECT--
string(5) "PRE_1"
string(5) "PRE_2"
string(4) "CALL"
string(6) "POST_2"
string(6) "POST_1"