--TEST--
Check if hooks are invoked only once for reimplemented interfaces
--EXTENSIONS--
otel
--SKIPIF--
<?php if (PHP_VERSION_ID < 80000) echo 'skip requires php 8+'; ?>
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.cli.enabled=1
otel.log.level=warn

--FILE--
<?php
interface A {
    function m(): void;
}
interface B extends A {
}
class C implements A, B {
    function m(): void {}
}

\OpenTelemetry\Instrumentation\hook(A::class, 'm', fn() => var_dump('PRE'), fn() => var_dump('POST'));

(new C)->m();
?>
--EXPECT--
string(3) "PRE"
string(4) "POST"
