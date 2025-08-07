--TEST--
zf1 route hook adds extra data to root span
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.log.level=warn
otel.cli.enabled=On
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\SpanExporter\Memory;
use OpenTelemetry\API\Trace\LocalRootSpan;

require_once __DIR__ . '/vendor/autoload.php';

$root = Globals::tracerProvider()
    ->getTracer('my_tracer', '0.1', 'schema.url')
    ->spanBuilder('root')
    ->setSpanKind(1)
    ->startSpan();
$scope = $root->activate();

$front = Zend_Controller_Front::getInstance();
$router = new Zend_Controller_Router_Rewrite();
$front->setRouter($router);

$request = new Zend_Controller_Request_Http('http://example.com/my-controller/my-action');
$front->setRequest($request);

$router->route($request);

$scope->detach();
$root->end();

var_dump(Memory::count());
$spans = Memory::getSpans();
$root_span = $spans[0];
var_dump($root_span);
?>
--EXPECTF--
int(1)
array(10) {
  ["name"]=>
  string(35) "GET default/my-controller/my-action"
  ["span_context"]=>
  array(4) {
    %A
  }
  ["parent_span_id"]=>
  string(16) "0000000000000000"
  ["span_kind"]=>
  string(6) "Server"
  ["start_time"]=>
  int(%d)
  ["end_time"]=>
  int(%d)
  ["instrumentation_scope"]=>
  array(4) {
    %A
  }
  ["status"]=>
  string(5) "Unset"
  ["attributes"]=>
  array(4) {
    ["php.framework.name"]=>
    string(3) "zf1"
    ["php.framework.module.name"]=>
    string(7) "default"
    ["php.framework.controller.name"]=>
    string(13) "my-controller"
    ["php.framework.action.name"]=>
    string(9) "my-action"
  }
  ["events"]=>
  array(0) {
  }
}