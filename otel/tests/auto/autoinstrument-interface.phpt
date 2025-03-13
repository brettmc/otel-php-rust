--TEST--
Autoinstrument an interface
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
interface IDemo {
    public function foo(): void;
}
class DemoClass implements IDemo {
    public function foo(): void
    {
        var_dump("foo");
    }
}

$demo = new DemoClass();
$demo->foo();
?>
--EXPECTF--
string(3) "foo"
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "php-auto-instrumentation"

	Name        : DemoClass::foo
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: %s
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset
	Attributes:
		 ->  code.function.name: String(Owned("DemoClass::foo"))
		 ->  code.file.path: String(Owned("%s"))
		 ->  code.line.number: I64(%d)
