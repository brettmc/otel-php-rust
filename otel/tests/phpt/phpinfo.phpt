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
otel.cli.create_root_span => 0 => 0
otel.cli.enable => 0 => 0
otel.dotenv.per_request => 0 => 0
otel.log.file => %s => %s
otel.log.level => error => error
%A