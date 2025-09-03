--TEST--
Check if pre hook can return $params for internal function
--SKIPIF--
<?php if (PHP_VERSION_ID < 80200) die('skip requires PHP >= 8.2'); ?>
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.cli.enabled=1
otel.log.level=warn

--FILE--
<?php
\OpenTelemetry\Instrumentation\hook(null, 'array_map', fn($obj, array $params) => $params);

array_map(var_dump(...), ['HELLO']);
?>
--EXPECT--
string(5) "HELLO"
