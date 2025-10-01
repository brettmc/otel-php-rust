--TEST--
Check if calling die or exit will finish gracefully
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

class TestClass {
    public static function countFunction()
    {
       for ($i = 1; $i <= 300; $i++) {
            if ($i === 200) {
                die('exit!');
            }
       }
    }
}

hook(
    'TestClass',
    'countFunction',
    null,
    static function ($object, array $params, $ret, \Throwable $exception ) {}
);

try{
    TestClass::countFunction();
}
catch(Exception $e) {}
// Comment out line below and revert fix in order to trigger segfault
// reproduction frequency depends on platform
catch(TypeError $t) {}
?>

--EXPECTF--
exit!%smessage=OpenTelemetry: post hook threw exception, class=TestClass function=countFunction message=%s: Argument #4 ($exception) must be of type Throwable, null given%s