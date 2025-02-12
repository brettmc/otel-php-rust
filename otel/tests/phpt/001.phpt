--TEST--
Check if module is loaded
--EXTENSIONS--
otel
--FILE--
<?php
var_dump(extension_loaded('otel'));
?>
--EXPECT--
bool(true)