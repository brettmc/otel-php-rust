--TEST--
Autoinstrument an internal function
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=1
otel.log.level=warn
--ENV--
OTEL_TRACES_EXPORTER=console
OTEL_SPAN_PROCESSOR=simple
--FILE--
<?php
var_dump(extension_loaded('otel'));
var_dump(phpversion());
?>
--EXPECTF--
bool(true)
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
string(%d) "%s"