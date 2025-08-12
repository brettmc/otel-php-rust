--TEST--
zf1 zend_db query span
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.log.level=warn
otel.cli.enabled=On
otel.cli.create_root_span=On
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

include __DIR__ . '/vendor/autoload.php';

$root = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url')->spanBuilder('root')->startSpan();
$scope = $root->activate();

$dbname = __DIR__ . '/data/test.sqlite';
$db = new Zend_Db_Adapter_Pdo_Sqlite(array('dbname' => $dbname));
$stmt = $db->prepare('select * from users');
var_dump(get_class($stmt));
$stmt->execute();
$result = $stmt->fetchAll();

$scope->detach();
$root->end();
assert(count($result) === 2, 'two rows should be returned');
assert(Memory::count() === 3, 'three spans should be created: root + prepare + execute');

$spans = Memory::getSpans();
//var_dump($spans);
$prepare_span = $spans[0];
$execute_span = $spans[1];
$root_span = $spans[2];

assert($execute_span['name'] === 'Statement::execute');
assert($prepare_span['name'] === 'Statement::prepare');
assert($root_span['name'] === 'root');

assert($prepare_span['span_context']['trace_id'] === $root_span['span_context']['trace_id']);
assert($execute_span['span_context']['trace_id'] === $root_span['span_context']['trace_id']);
assert($prepare_span['parent_span_id'] === $root_span['span_context']['span_id']);
assert($execute_span['parent_span_id'] === $root_span['span_context']['span_id']);
var_dump($execute_span['attributes']);
//@todo link between execute and prepare
var_dump($prepare_span['attributes']);
?>
--EXPECTF--
%Astring(21) "Zend_Db_Statement_Pdo"
array(3) {
  ["code.function.name"]=>
  string(26) "Zend_Db_Statement::execute"
  ["code.file.path"]=>
  string(%d) "%s/Zend/Db/Statement.php"
  ["code.line.number"]=>
  int(%d)
}
array(4) {
  ["code.function.name"]=>
  string(37) "Zend_Db_Adapter_Pdo_Abstract::prepare"
  ["code.file.path"]=>
  string(%d) "%s/Zend/Db/Adapter/Pdo/Abstract.php"
  ["code.line.number"]=>
  int(%d)
  ["db.query.text"]=>
  string(19) "select * from users"
}