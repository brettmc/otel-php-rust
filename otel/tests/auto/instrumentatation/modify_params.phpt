--TEST--
Check if pre hook can modify params of function
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
          0 => null,  //make first param null
          2 => 'baz', //replace 3rd param
          3 => 'bat', //add 4th param
        ];
    }
);
function hello($one = null, $two = null, $three = null, $four = null) {
  var_dump(func_get_args());
}

hello('a', 'b', 'c');
?>
--EXPECT--
array(4) {
  [0]=>
  NULL
  [1]=>
  string(1) "b"
  [2]=>
  string(3) "baz"
  [3]=>
  string(3) "bat"
}
