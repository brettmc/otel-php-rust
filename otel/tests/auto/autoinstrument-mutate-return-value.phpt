--TEST--
Autoinstrument a class, mutating return value
--EXTENSIONS--
otel
--SKIPIF--
<?php
if (PHP_VERSION_ID < 70200) {
    die("skip requires PHP 7.2+");
}
--ENV--
OTEL_TRACES_EXPORTER=none
--INI--
otel.cli.enabled=1
otel.log.level="warn"
--FILE--
<?php
class DemoClass {
    public function hello()
    {
        return 'hello';
    }
}

$demo = new DemoClass();
var_dump($demo->hello());
?>
--EXPECT--
string(7) "goodbye"