--TEST--
Autoinstrument a function twice

--DESCRIPTION--
Test caching of instrumentation decisions when using zend_execute_ex
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=none
--SKIPIF--
<?php
if (!defined('OTEL_AUTO_INSTRUMENTATION') || OTEL_AUTO_INSTRUMENTATION !== 'zend_execute_ex') {
    die("skip requires zend_execute_ex instrumentation");
}
--INI--
otel.log.level="warn"
otel.log.file="/dev/stdout"
--FILE--
<?php
function demoFunction() {
    var_dump("demo_function");
}

demoFunction();
demoFunction();
?>
--EXPECTF--
string(13) "demo_function"
string(13) "demo_function"
