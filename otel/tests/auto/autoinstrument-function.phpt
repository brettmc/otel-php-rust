--TEST--
Autoinstrument a function
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
function demoFunction() {
    var_dump("demo_function");
}

demoFunction();
?>
--EXPECTF--
string(13) "demo_function"
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "php-auto-instrumentation"

	Name        : i-was-renamed
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: %s
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset
	Attributes:
		 ->  code.function.name: String(Owned("demoFunction"))
		 ->  code.file.path: String(Owned("/usr/src/myapp/tests/auto/autoinstrument-function.php"))
		 ->  code.line.number: I64(%d)
		 ->  my-attribute: String(Owned("my-value"))