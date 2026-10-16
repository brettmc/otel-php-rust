--TEST--
Set and retrieve values from context, by key identity
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=none
OTEL_LOGS_EXPORTER=none
OTEL_METRICS_EXPORTER=none
--FILE--
<?php
// The basics of keyed storage: a value written under one key is readable back under that
// same key, and under no other. `with()` takes a ContextKeyInterface, as upstream does, so
// a key is an object and not a string -- two keys are distinct even when their names match.
use OpenTelemetry\Context\Context;

$some = Context::createKey('some_key');
$another = Context::createKey('another_key');

$context = Context::getCurrent()->with($some, 'A');
var_dump($context->get($some));

// Two keys coexist: the second write must not displace the first.
$context = $context->with($another, 'B');
var_dump($context->get($another));
var_dump($context->get($some));

// An absent key is null, not the most recent value under any key.
var_dump($context->get(Context::createKey('never_set')));

// Values are not restricted to strings.
$objectKey = Context::createKey('object');
$object = new stdClass();
$object->n = 1;
$context = $context->with($objectKey, $object);
var_dump($context->get($objectKey) === $object);
$arrayKey = Context::createKey('array');
$context = $context->with($arrayKey, ['a' => 1]);
var_dump($context->get($arrayKey));
?>
--EXPECT--
string(1) "A"
string(1) "B"
string(1) "A"
NULL
bool(true)
array(1) {
  ["a"]=>
  int(1)
}
