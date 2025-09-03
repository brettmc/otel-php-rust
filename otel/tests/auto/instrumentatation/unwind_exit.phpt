--TEST--
Test UnwindExit from die/exit is not exposed to userland code
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

class TestClass {
    public static function run(): void
    {
       die('exit!');
    }
}

hook(
    'TestClass',
    'run',
    null,
    static function ($object, array $params, mixed $ret, ?\Throwable $exception ) {
      echo PHP_EOL . 'post';
    }
);

TestClass::run();
?>

--EXPECT--
exit!
post