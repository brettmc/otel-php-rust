--TEST--
Test laminas db prepare and execute query
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
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$tracer = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url');
$root = $tracer->spanBuilder('root')->startSpan();
$scope = $root->activate();

$adapter = new Adapter([
    'driver'   => 'Pdo_Sqlite',
    'database' => __DIR__ . '/data/test.sqlite',
]);

$sql    = new Sql($adapter);
$select = $sql->select('users');
$statement = $sql->prepareStatementForSqlObject($select);
$result    = $statement->execute();

$root->end();
$scope->detach();

var_dump(Memory::count());
$spans = Memory::getSpans();
//var_dump(Memory::getSpans());
$connectSpan = $spans[0];
$prepareSpan = $spans[1];
$executeSpan = $spans[2];
$rootSpan = $spans[3];

var_dump($connectSpan['name']);
var_dump($prepareSpan['name']);
var_dump($executeSpan['name']);
echo '=== Connection ===' . PHP_EOL;
var_dump($connectSpan['attributes']);
echo '=== Prepare ===' . PHP_EOL;
var_dump($prepareSpan['attributes']);
echo '=== Execute ===' . PHP_EOL;
var_dump($executeSpan['attributes']);
?>
--EXPECTF--
int(4)
string(%d) "Db::connect"
string(%d) "Sql::prepare"
string(%d) "Statement::execute"
=== Connection ===
array(%d) {
%A
  ["db.namespace"]=>
  string(%d) "%s/test.sqlite"
  ["db.system.name"]=>
  string(%d) "sqlite"
}
=== Prepare ===
array(%d) {
%A
  ["db.query.text"]=>
  string(%d) " SELECT "users".* FROM "users""
}
=== Execute ===
array(%d) {
%A
}