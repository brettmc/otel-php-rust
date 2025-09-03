--TEST--
Check if pre hook can try to modify invalid named params of function
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.cli.enabled=1
otel.log.level=warn

--FILE--
<?php
OpenTelemetry\Instrumentation\hook(
    null,
    'hello',
    function($obj, array $params) {
        return [
          'four' => 'replaced',
        ];
    }
);
function hello($one = null, $two = null, $three = null) {
  var_dump(func_get_args());
}

hello('a', 'b', 'c');
?>
--EXPECTF--
Warning: hello(): OpenTelemetry: pre hook unknown named arg four, class=null function=hello in %s
array(3) {
  [0]=>
  string(1) "a"
  [1]=>
  string(1) "b"
  [2]=>
  string(1) "c"
}
