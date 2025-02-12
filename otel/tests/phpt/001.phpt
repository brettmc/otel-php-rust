--TEST--
Check if module is loaded
--FILE--
<?php
var_dump(extension_loaded('otel'));
?>
--EXPECT--
bool(true)