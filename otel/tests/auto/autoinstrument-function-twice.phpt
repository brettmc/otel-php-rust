--TEST--
Autoinstrument a function twice (zend_execute_ex)
--DESCRIPTION--
Test caching of instrumentation decisions when using zend_execute_ex
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=none
--SKIPIF--
<?php
if (PHP_VERSION_ID >= 80000) {
    die("skip requires PHP 7.x");
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
