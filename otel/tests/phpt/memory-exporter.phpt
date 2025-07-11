--TEST--
Test memory exporter
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.log.level="warn"
otel.log.file="/dev/stdout"
otel.cli.enable=1
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\SpanExporter\Memory;
$tracer = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url');
var_dump(Memory::count());
$tracer->spanBuilder('root')->startSpan()->end();
var_dump(Memory::count());
var_dump(Memory::getSpans());
Memory::reset();
var_dump(Memory::count());
var_dump(Memory::getSpans());
?>
--EXPECTF--
int(0)
int(1)
array(1) {
  [0]=>
  array(10) {
    ["name"]=>
    string(4) "root"
    ["span_context"]=>
    array(4) {
      ["trace_id"]=>
      string(32) "%s"
      ["span_id"]=>
      string(16) "%s"
      ["trace_flags"]=>
      string(2) "01"
      ["is_remote"]=>
      bool(false)
    }
    ["parent_span_id"]=>
    string(16) "0000000000000000"
    ["span_kind"]=>
    string(8) "Internal"
    ["start_time"]=>
    int(%d)
    ["end_time"]=>
    int(%d)
    ["instrumentation_scope"]=>
    array(4) {
      ["name"]=>
      string(9) "my_tracer"
      ["version"]=>
      string(3) "0.1"
      ["schema_url"]=>
      string(10) "schema.url"
      ["attributes"]=>
      array(0) {
      }
    }
    ["status"]=>
    string(5) "Unset"
    ["attributes"]=>
    array(0) {
    }
    ["events"]=>
    array(0) {
    }
  }
}
int(0)
array(0) {
}