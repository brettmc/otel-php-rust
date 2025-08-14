--TEST--
Test zf1 Zend_Db query
--EXTENSIONS--
otel
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

use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$tracer = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url');
$root = $tracer->spanBuilder('root')->startSpan();
$scope = $root->activate();

$dbname = __DIR__ . '/data/test.sqlite';
$db = new Zend_Db_Adapter_Pdo_Sqlite(array('dbname' => $dbname));
$stmt = $db->prepare('select * from users');
$stmt->execute();

$root->end();
$scope->detach();

var_dump(Memory::count());
$spans = Memory::getSpans();
$prepareSpan = $spans[0];
var_dump($prepareSpan['name']);
var_dump($prepareSpan['attributes']);
?>
--EXPECTF--
%Aint(3)
string(18) "Statement::prepare"
array(%d) {
%A
  ["db.query.text"]=>
  string(19) "select * from users"
}