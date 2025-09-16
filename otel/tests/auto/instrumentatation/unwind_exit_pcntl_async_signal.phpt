--TEST--
Test UnwindExit caused by async pcntl handler is not suppressed
--EXTENSIONS--
otel
pcntl
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.cli.enabled=1
otel.log.level=warn
--FILE--
<?php

use function OpenTelemetry\Instrumentation\hook;

hook(
    null,
    'sleep',
    static fn() => null,
    static fn() => null,
);

pcntl_async_signals(true);
pcntl_signal(SIGALRM, static function() {
    echo "timeout\n";
    exit(1);
});

pcntl_alarm(1);
sleep(2);
echo "fail\n";
?>

--EXPECT--
timeout
