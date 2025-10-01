--TEST--
Test UnwindExit caused by async pcntl handler does not leak otel_exception_state memory
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.cli.enabled=1
otel.log.level=warn
--XFAIL--
not implemented
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
        for (;;) {}
    }
);

pcntl_async_signals(true);
pcntl_signal(SIGALRM, static function() {
    echo "timeout\n";
    exit(1);
});

pcntl_alarm(1);
throwingFunction();
echo "fail\n";
?>

--EXPECT--
timeout
