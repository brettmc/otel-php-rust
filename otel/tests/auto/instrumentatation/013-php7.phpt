--TEST--
Check if hooks are invoked for closures
--EXTENSIONS--
otel
--SKIPIF--
<?php if (PHP_VERSION_ID >= 80000) echo 'skip requires php 7.x'; ?>
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
    function() { var_dump('PRE');},
    function() { var_dump('POST');}
);

function helloWorld() {
    var_dump('HELLO');
}

$closure = function() {
    helloWorld();
};
$closure();
?>
--EXPECT--
string(3) "PRE"
string(5) "HELLO"
string(4) "POST"