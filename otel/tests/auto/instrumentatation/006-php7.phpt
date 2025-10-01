--TEST--
Check if hooks receives arguments and return value
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
\OpenTelemetry\Instrumentation\hook(
    null,
    'helloWorld',
    function() { var_dump(func_get_args());},
    function() { var_dump(func_get_args());}
);

function helloWorld(string $a) {
    return 42;
}
//in PHP 7.x, if the return value is not used, it is optimized away (and therefore not seen by post hooks)
$return = helloWorld('a');
?>
--EXPECTF--
array(8) {
  [0]=>
  NULL
  [1]=>
  array(1) {
    [0]=>
    string(1) "a"
  }
  [2]=>
  NULL
  [3]=>
  string(10) "helloWorld"
  [4]=>
  string(%d) "%s/006-php7.php"
  [5]=>
  int(%d)
  [6]=>
  array(0) {
  }
  [7]=>
  array(0) {
  }
}
array(8) {
  [0]=>
  NULL
  [1]=>
  array(1) {
    [0]=>
    string(1) "a"
  }
  [2]=>
  int(42)
  [3]=>
  NULL
  [4]=>
  NULL
  [5]=>
  string(10) "helloWorld"
  [6]=>
  string(%d) "%s/006-php7.php"
  [7]=>
  int(%d)
}