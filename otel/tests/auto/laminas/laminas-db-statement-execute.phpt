--TEST--
Test laminas db statement prepare+execute
--EXTENSIONS--
otel
--SKIPIF--
<?php
if (PHP_VERSION_ID < 70100 || PHP_VERSION_ID >= 80400) {
    die('skip requires PHP 7.1 -> 8.3');
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

// method 1
$sql = 'select * from users';
$statement = $adapter->createStatement($sql);
$statement->prepare();
$result    = $statement->execute();

// method 2
$statement = $adapter->createStatement();
$statement->prepare($sql);
$result = $statement->execute();

// method 3
$result = $adapter->query($sql, Adapter::QUERY_MODE_EXECUTE);

var_dump(Memory::count());
echo '===first span===' . PHP_EOL;
$first = Memory::getSpans()[0];
var_dump($first['name']);
var_dump($first['span_kind']);
var_dump($first['attributes']);

echo '===second span===' . PHP_EOL;
$second = Memory::getSpans()[1];
var_dump($second['name']);
var_dump($second['span_kind']);
var_dump($second['attributes']);
?>
--EXPECTF--
int(3)
===first span===
string(%d) "SELECT users"
string(%d) "Client"
array(%d) {
  ["code.function.name"]=>
  string(%d) "%s\Statement::execute"
  ["code.file.path"]=>
  string(%d) "%s/Adapter/Driver/Pdo/Statement.php"
  ["code.line.number"]=>
  int(%d)
  ["db.query.text"]=>
  string(%d) "select * from users"
}
===second span===
string(%d) "SELECT users"
string(%d) "Client"
array(%d) {
  ["code.function.name"]=>
  string(%d) "%s\Statement::execute"
  ["code.file.path"]=>
  string(%d) "%s/Adapter/Driver/Pdo/Statement.php"
  ["code.line.number"]=>
  int(%d)
  ["db.query.text"]=>
  string(%d) "select * from users"
}