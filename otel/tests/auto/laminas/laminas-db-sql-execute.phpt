--TEST--
Test laminas db sql prepare+execute
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

$sql       = new Sql($adapter);
$select    = $sql->select()->from('users')->where(['id' => 1]);
$statement = $sql->prepareStatementForSqlObject($select);
$result    = $statement->execute();

var_dump(Memory::count());
$span = Memory::getSpans()[0];
var_dump($span['name']);
var_dump($span['span_kind']);
var_dump($span['attributes']);
?>
--EXPECTF--
int(1)
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
  string(%d) " SELECT "users".* FROM "users" WHERE %s"
  ["db.namespace"]=>
  string(%d) "%s/test.sqlite"
  ["db.system.name"]=>
  string(6) "sqlite"
}