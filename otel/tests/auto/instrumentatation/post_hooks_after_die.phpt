--TEST--
Calling die/exit still executes post hooks
--EXTENSIONS--
otel
--SKIPIF--
<?php if (version_compare(PHP_VERSION, '8.0.0', '<')) echo 'skip requires php 8.0+'; ?>
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.cli.enabled=1
otel.log.level=warn

--FILE--

<?php

use function OpenTelemetry\Instrumentation\hook;

function goodbye() {
    var_dump('goodbye');
    die;
}

\OpenTelemetry\Instrumentation\hook(null, 'goodbye', fn() => var_dump('PRE'), fn() => var_dump('POST'));

goodbye();
?>

--EXPECT--
string(3) "PRE"
string(7) "goodbye"
string(4) "POST"