--TEST--
Test laminas db statement prepare+execute span name for db operations
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

//insert
$statement = $adapter->createStatement();
$statement->prepare('insert into "scratch" (i) values (1)');
$result = $statement->execute();

//update
$statement = $adapter->createStatement();
$statement->prepare('update `scratch` set i=0 where i=1');
$result = $statement->execute();

//delete
$statement = $adapter->createStatement();
$statement->prepare('delete from scratch where i=0');
$result = $statement->execute();

//"other"
$statement = $adapter->createStatement();
$statement->prepare('vacuum');
$result = $statement->execute();

var_dump(Memory::count());
$spans = Memory::getSpans();
$insert = $spans[0];
$update = $spans[1];
$delete = $spans[2];
$vacuum = $spans[3];

var_dump($insert['name']);
var_dump($update['name']);
var_dump($delete['name']);
var_dump($vacuum['name']);
?>
--EXPECTF--
int(4)
string(14) "INSERT scratch"
string(14) "UPDATE scratch"
string(14) "DELETE scratch"
string(5) "OTHER"