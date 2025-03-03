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
opentelemetry-rust => %s

Directive => Local Value => Master Value
otel.log_level => error => error
%A