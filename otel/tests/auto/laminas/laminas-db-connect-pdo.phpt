--TEST--
Test laminas db connect
--EXTENSIONS--
otel
--SKIPIF--
<?php
if (PHP_VERSION_ID < 70100 || PHP_VERSION_ID >= 80300) {
    die('skip requires PHP 7.1 -> 8.2');
}
?>
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.log.level="warn"
otel.log.file="/dev/stdout"
otel.cli.enabled=1
--FILE--
<?php
require_once __DIR__ . '/vendor/autoload.php';

use Laminas\Db\Adapter\Adapter;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$adapter = new Adapter([
    'driver'   => 'pdo',
    'dsn'      => 'sqlite:' . __DIR__ . '/data/test.sqlite',
]);
$connection = $adapter->getDriver()->getConnection()->connect();

var_dump(Memory::count());
$connect = Memory::getSpans()[0];
var_dump($connect['attributes']);
?>
--EXPECTF--
int(1)
array(%d) {
%A
  ["db.system.name"]=>
  string(6) "sqlite"
}