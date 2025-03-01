--TEST--
Check phpinfo
--EXTENSIONS--
otel
--FILE--
<?php
phpinfo();
?>
--EXPECTF--
%A
otel

version => %s
%A