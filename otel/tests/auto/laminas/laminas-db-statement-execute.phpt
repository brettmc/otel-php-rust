--TEST--
Test laminas db statement execute
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
$sql = 'select * from users';

// method 1 - create statement with sql
$statement = $adapter->createStatement($sql);
$statement->prepare();
$result    = $statement->execute();

// method 2 - empty statement, prepare with sql
$statement = $adapter->createStatement();
$statement->prepare($sql);
$result = $statement->execute();

// method 3 - query with params
$result = $adapter->query($sql, []);

// method 4 - query with direct execute
$result = $adapter->query($sql, Adapter::QUERY_MODE_EXECUTE);
//var_dump(Memory::getSpans());
//die;

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

echo '===third span===' . PHP_EOL;
$third = Memory::getSpans()[2];
var_dump($third['name']);
var_dump($third['span_kind']);
var_dump($third['attributes']);

echo '===fourth span===' . PHP_EOL;
$fourth = Memory::getSpans()[3];
var_dump($fourth['name']);
var_dump($fourth['span_kind']);
var_dump($fourth['attributes']);
?>
--EXPECTF--
int(4)
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
===third span===
string(12) "SELECT users"
string(6) "Client"
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
===fourth span===
string(12) "SELECT users"
string(6) "Client"
array(4) {
  ["code.function.name"]=>
  string(%d) "Laminas\Db\Adapter\Driver\Pdo\Connection::execute"
  ["code.file.path"]=>
  string(%d) "%s/Adapter/Driver/Pdo/Connection.php"
  ["code.line.number"]=>
  int(%d)
  ["db.query.text"]=>
  string(%d) "select * from users"
}