--TEST--
Test UnwindExit caused by async pcntl handler is not suppressed
--EXTENSIONS--
otel
pcntl
--SKIPIF--
<?php if (PHP_VERSION_ID >= 80000 || !function_exists('pcntl_async_signals')) echo 'skip requires php 7.x and pcntl_async_signals'; ?>
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
    static function() {},
    static function() {}
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
