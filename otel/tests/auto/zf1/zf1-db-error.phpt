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

use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$tracer = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url');
$root = $tracer->spanBuilder('root')->startSpan();
$scope = $root->activate();

$dbname = __DIR__ . '/data/test.sqlite';
$db = new Zend_Db_Adapter_Pdo_Sqlite(array('dbname' => $dbname));
try {
    $stmt = $db->prepare('select * from foo');
    $stmt->execute();
} catch (Exception $e) {
    // do nothing
}

$root->end();
$scope->detach();

var_dump(Memory::count());
$spans = Memory::getSpans();
$prepareSpan = $spans[0];
var_dump($prepareSpan['name']);
var_dump($prepareSpan['status']);
var_dump($prepareSpan['attributes']);
var_dump($prepareSpan['events']);
?>
--EXPECTF--
%Aint(2)
string(%d) "Statement::prepare"
string(%d) "Error { description: "SQLSTATE[HY000]: General error: 1 no such table: foo" }"
array(%d) {
%A
  ["db.query.text"]=>
  string(17) "select * from foo"
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
      string(52) "SQLSTATE[HY000]: General error: 1 no such table: foo"
      ["exception.type"]=>
      string(27) "Zend_Db_Statement_Exception"
      ["exception.stacktrace"]=>
      string(%d) "%A"
    }
  }
}