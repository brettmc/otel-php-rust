--TEST--
Check if post hook can returned modified clone
--DESCRIPTION--
A different object might be returned than the one provided to post hook. For example, PSR-7 messages are immutable and modifying
one creates a new instance.
--EXTENSIONS--
otel
--SKIPIF--
<?php if (PHP_VERSION_ID >= 80000) echo 'skip requires php 7.x'; ?>
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.cli.enabled=1
otel.log.level=warn

--FILE--
<?php
class Foo
{
    public $a = null;
    public function __construct($a = null)
    {
        $this->a = $a;
    }
    public function modify($value): Foo
    {
        $new = clone($this);
        $new->a = $value;

        return $new;
    }
}

\OpenTelemetry\Instrumentation\hook(null, 'getFoo', null, function($obj, array $params, Foo $foo): Foo {
    return $foo->modify('b');
});

function getFoo(): Foo {
    return new Foo('a');
}

var_dump(getFoo());
?>
--EXPECTF--
object(Foo)#%d (1) {
  ["a"]=>
  string(1) "b"
}
