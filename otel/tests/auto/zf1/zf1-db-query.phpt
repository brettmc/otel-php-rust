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

use OpenTelemetry\API\Trace\SpanExporter\Memory;

$dbname = __DIR__ . '/data/test.sqlite';
$db = new Zend_Db_Adapter_Pdo_Sqlite(array('dbname' => $dbname));
$stmt = $db->prepare('select * from users');
$stmt->execute();

var_dump(Memory::count());
$spans = Memory::getSpans();
$prepare = $spans[0];
var_dump($prepare['name']);
var_dump($prepare['attributes']);

$execute = $spans[1];
var_dump($execute['name']);
var_dump($execute['attributes']);

assert(count($execute['links']) === 1);
assert($execute['links'][0]['span_context']['span_id'] === $prepare['span_context']['span_id']);
?>
--EXPECTF--
int(2)
string(20) "prepare SELECT users"
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
string(12) "SELECT users"
array(4) {
  ["code.function.name"]=>
  string(26) "Zend_Db_Statement::execute"
  ["code.file.path"]=>
  string(%d) "%s/library/Zend/Db/Statement.php"
  ["code.line.number"]=>
  int(%d)
  ["db.query.text"]=>
  string(19) "select * from users"
}