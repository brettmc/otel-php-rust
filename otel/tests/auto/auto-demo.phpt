--TEST--
Autoinstrument a class + function
--EXTENSIONS--
otel
--ENV--
OTEL_EXPORTER_OTLP_PROTOCOL=console
--FILE--
<?php
class DemoClass {
    function test(): void
    {
        var_dump("test");
    }
}

$demo = new DemoClass();
$demo->test();
?>
--EXPECT--
string(4) "test"
