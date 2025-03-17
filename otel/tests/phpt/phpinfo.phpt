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

Directive => Local Value => Master Value
otel.log.file => %s => %s
otel.log.level => error => error
%A