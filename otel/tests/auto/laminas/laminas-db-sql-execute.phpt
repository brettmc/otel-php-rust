--TEST--
Test laminas db sql prepare+execute
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

$sql       = new Sql($adapter);
$select    = $sql->select()->from('users')->where(['id' => 1]);
$statement = $sql->prepareStatementForSqlObject($select);
$result    = $statement->execute();

var_dump(Memory::count()); //expect 3: connect, prepare, execute
echo '===connect===' . PHP_EOL;
$connect = Memory::getSpans()[0];
var_dump($connect['name']);
var_dump($connect['span_kind']);
var_dump($connect['attributes']);
echo '===prepare===' . PHP_EOL;
$prepare = Memory::getSpans()[1];
var_dump($prepare['name']);
var_dump($prepare['span_kind']);
var_dump($prepare['attributes']);
echo '===execute===' . PHP_EOL;
$execute = Memory::getSpans()[2];
var_dump($execute['name']);
var_dump($execute['span_kind']);
var_dump($execute['attributes']);
?>
--EXPECTF--
int(3)
===connect===
string(7) "connect"
string(6) "Client"
array(5) {
  ["code.function.name"]=>
  string(49) "Laminas\Db\Adapter\Driver\Pdo\Connection::connect"
  ["code.file.path"]=>
  string(%d) "%s/Adapter/Driver/Pdo/Connection.php"
  ["code.line.number"]=>
  int(%d)
  ["db.namespace"]=>
  string(%d) "%s/test.sqlite"
  ["db.system.name"]=>
  string(6) "sqlite"
}
===prepare===
string(20) "prepare SELECT users"
string(6) "Client"
array(6) {
  ["code.function.name"]=>
  string(48) "Laminas\Db\Adapter\Driver\Pdo\Statement::prepare"
  ["code.file.path"]=>
  string(%d) "%s/Adapter/Driver/Pdo/Statement.php"
  ["code.line.number"]=>
  int(%d)
  ["db.query.text"]=>
  string(%d) " SELECT %s FROM %s WHERE %s"
  ["db.namespace"]=>
  string(%d) "%s/test.sqlite"
  ["db.system.name"]=>
  string(6) "sqlite"
}
===execute===
string(%d) "SELECT users"
string(6) "Client"
array(6) {
  ["code.function.name"]=>
  string(48) "Laminas\Db\Adapter\Driver\Pdo\Statement::execute"
  ["code.file.path"]=>
  string(%d) "%s/Adapter/Driver/Pdo/Statement.php"
  ["code.line.number"]=>
  int(%d)
  ["db.query.text"]=>
  string(%d) " SELECT %s FROM %s WHERE %s"
  ["db.namespace"]=>
  string(%d) "%s/test.sqlite"
  ["db.system.name"]=>
  string(6) "sqlite"
}