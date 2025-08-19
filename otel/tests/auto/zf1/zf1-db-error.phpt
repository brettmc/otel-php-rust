--TEST--
Test zf1 Zend_Db query error
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
try {
    $stmt = $db->prepare('select * from does_not_exist');
    $stmt->execute();
} catch (Exception $e) {
    // do nothing
}

var_dump(Memory::count());
$spans = Memory::getSpans();
$prepareSpan = $spans[1];
var_dump($prepareSpan['name']);
var_dump($prepareSpan['status']);
var_dump($prepareSpan['attributes']);
var_dump($prepareSpan['events']);
?>
--EXPECTF--
%Aint(2)
string(%d) "prepare SELECT does_not_exist"
string(%d) "Error { description: "%s no such table: does_not_exist" }"
array(%d) {
%A
  ["db.query.text"]=>
  string(%d) "select * from does_not_exist"
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
      string(27) "Zend_Db_Statement_Exception"
      ["exception.stacktrace"]=>
      string(%d) "%A"
    }
  }
}