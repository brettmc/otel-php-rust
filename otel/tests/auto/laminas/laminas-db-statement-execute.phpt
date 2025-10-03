--TEST--
Test laminas db statement execute
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

var_dump(Memory::count());
foreach (Memory::getSpans() as $span) {
    var_dump($span['name']);
}

echo '===direct execute===' . PHP_EOL;
//validate the direct execute span
$execute_span = Memory::getSpans()[7];
var_dump($execute_span['name']);
var_dump($execute_span['span_kind']);
var_dump($execute_span['attributes']);
?>
--EXPECTF--
int(8)
string(7) "connect"
string(20) "prepare SELECT users"
string(12) "SELECT users"
string(20) "prepare SELECT users"
string(12) "SELECT users"
string(20) "prepare SELECT users"
string(12) "SELECT users"
string(12) "SELECT users"
===direct execute===
string(12) "SELECT users"
string(6) "Client"
array(4) {
  ["code.function.name"]=>
  string(49) "Laminas\Db\Adapter\Driver\Pdo\Connection::execute"
  ["code.file.path"]=>
  string(%d) "%s/Adapter/Driver/Pdo/Connection.php"
  ["code.line.number"]=>
  int(%d)
  ["db.query.text"]=>
  string(19) "select * from users"
}