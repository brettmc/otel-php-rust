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
otel.auto.disabled_plugins => no value => no value
otel.auto.enabled => 1 => 1
otel.cli.create_root_span => 0 => 0
otel.cli.enabled => 0 => 0
otel.dotenv.per_request => 0 => 0
otel.log.file => %s => %s
otel.log.level => error => error
%A