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
otel.log.file => /var/log/ext-otel.log => /var/log/ext-otel.log
otel.log.level => error => error
%A