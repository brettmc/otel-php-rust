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
$stmt = $db->prepare('select * from users where id = :id');
$stmt->execute(['id' => 1]);
$stmt->execute(['id' => 2]);

var_dump(Memory::count());
$spans = Memory::getSpans();
foreach ($spans as $span) {
    var_dump($span['name']);
}
$connect = $spans[0];
$prepare = $spans[1];
$execute_one = $spans[2];
$execute_two = $spans[3];

assert(count($prepare['links']) === 1);
assert($prepare['links'][0]['span_context']['span_id'] === $connect['span_context']['span_id']);

assert(count($execute_one['links']) === 1);
assert($execute_one['links'][0]['span_context']['span_id'] === $prepare['span_context']['span_id']);

assert(count($execute_two['links']) === 1);
assert($execute_two['links'][0]['span_context']['span_id'] === $prepare['span_context']['span_id']);
?>
--EXPECTF--
%Aint(4)
string(7) "connect"
string(20) "prepare SELECT users"
string(12) "SELECT users"
string(12) "SELECT users"