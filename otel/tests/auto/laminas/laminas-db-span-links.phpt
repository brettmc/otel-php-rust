--TEST--
Test laminas db span linking
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
use Laminas\Db\Sql\Sql;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$adapter = new Adapter([
    'driver'   => 'Pdo_Sqlite',
    'database' => __DIR__ . '/data/test.sqlite',
]);
$sql = 'select * from users';

$statement = $adapter->createStatement($sql);
$statement->prepare();
$result    = $statement->execute();

var_dump(Memory::count());
$connect = Memory::getSpans()[0];
$prepare = Memory::getSpans()[1];
$execute = Memory::getSpans()[2];

// prepare is linked to connect span
assert(count($prepare['links']) === 1);
assert($prepare['links'][0]['span_context']['span_id'] === $connect['span_context']['span_id']);

// execute is linked to prepare span
assert(count($execute['links']) === 1);
assert($execute['links'][0]['span_context']['span_id'] === $prepare['span_context']['span_id']);

?>
--EXPECTF--
int(3)