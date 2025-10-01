--TEST--
Test UnwindExit in post handler does not leak otel_exception_state memory
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

use function OpenTelemetry\Instrumentation\hook;

function throwingFunction() {
    throw new Exception();
}

hook(
    null,
    'throwingFunction',
    static fn() => null,
    static function() {
        exit;
    }
);


throwingFunction();
echo "fail\n";
?>

--EXPECT--
