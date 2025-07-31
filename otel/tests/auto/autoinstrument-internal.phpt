--TEST--
Autoinstrument an internal function
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=1
--ENV--
OTEL_TRACES_EXPORTER=console
--SKIPIF--
<?php
if (PHP_VERSION_ID < 80200) {
    die("skip requires PHP 8.2");
}
--FILE--
<?php
var_dump(phpversion());
?>
--EXPECTF--
string(%d) "%s"
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "php-auto-instrumentation"

	Name        : phpversion
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: %s
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset
	Attributes:
		 ->  code.function.name: String(Owned("phpversion"))
