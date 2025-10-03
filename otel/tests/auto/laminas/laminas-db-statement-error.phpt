--TEST--
Test laminas db statement error
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

try {
    $statement = $adapter->createStatement();
    $statement->prepare('select * from does_not_exist');
} catch (\Exception $e) {
    var_dump($e->getMessage());
}
var_dump(Memory::count());
echo '===connect===' . PHP_EOL;
$connect = Memory::getSpans()[0];
var_dump($connect['name']);
echo '===prepare===' . PHP_EOL;
$prepare = Memory::getSpans()[1];
var_dump($prepare['name']);
var_dump($prepare['span_kind']);
var_dump($prepare['status']);
var_dump($prepare['attributes']);
var_dump($prepare['events']);
?>
--EXPECTF--
string(%d) "%s no such table: does_not_exist"
int(2)
===connect===
string(7) "connect"
===prepare===
string(%d) "prepare SELECT does_not_exist"
string(6) "Client"
string(%d) "Error { description: "%s no such table: does_not_exist" }"
array(6) {
  ["code.function.name"]=>
  string(48) "Laminas\Db\Adapter\Driver\Pdo\Statement::prepare"
  ["code.file.path"]=>
  string(%d) "%s/Adapter/Driver/Pdo/Statement.php"
  ["code.line.number"]=>
  int(%d)
  ["db.query.text"]=>
  string(28) "select * from does_not_exist"
  ["db.namespace"]=>
  string(%d) "%s/test.sqlite"
  ["db.system.name"]=>
  string(6) "sqlite"
}
array(1) {
  [0]=>
  array(3) {
    ["name"]=>
    string(9) "exception"
    ["timestamp"]=>
    int(%d)
    ["attributes"]=>
    array(3) {
      ["exception.message"]=>
      string(%d) "%s no such table: does_not_exist"
      ["exception.type"]=>
      string(12) "PDOException"
      ["exception.stacktrace"]=>
%A
    }
  }
}
