--TEST--
Test laminas db statement prepare+execute span name for db operations
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

$adapter->getDriver()->getConnection()->beginTransaction();

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

$adapter->getDriver()->getConnection()->rollback();

//"other" - any statement the span-name regexes in auto/utils.rs don't recognise lands
//here, so this only needs to be something that isn't SELECT/INSERT/UPDATE/DELETE. It must
//also be read-only: `vacuum` names the same span but rewrites the database file, which left
//the committed fixture showing as modified in git after every test run. The writes above are
//safe because they roll back, and a rolled-back transaction leaves the file byte-identical.
$statement = $adapter->createStatement();
$statement->prepare('pragma user_version');
$result = $statement->execute();

var_dump(Memory::count());
$spans = Memory::getSpans();
foreach ($spans as $span) {
    var_dump($span['name']);
}
?>
--EXPECTF--
int(9)
string(7) "connect"
string(22) "prepare INSERT scratch"
string(14) "INSERT scratch"
string(22) "prepare UPDATE scratch"
string(14) "UPDATE scratch"
string(22) "prepare DELETE scratch"
string(14) "DELETE scratch"
string(13) "prepare OTHER"
string(5) "OTHER"