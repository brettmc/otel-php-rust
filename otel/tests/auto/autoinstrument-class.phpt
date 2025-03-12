--TEST--
Autoinstrument a class + function
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
class DemoClass {
    public function test(): void
    {
        var_dump("test");
        $this->inner();
    }
    private function inner(): void
    {
        var_dump("inner");
    }
}

$demo = new DemoClass();
$demo->test();
?>
--EXPECTF--
string(4) "test"
string(5) "inner"
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "php-auto-instrumentation"

	Name        : DemoClass::inner
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: %s
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset
	Attributes:
		 ->  code.function.name: String(Owned("DemoClass::inner"))
		 ->  code.file.path: String(Owned("%s"))
		 ->  code.line.number: I64(%d)
Span #1
	Instrumentation Scope
		Name         : "php-auto-instrumentation"

	Name        : DemoClass::test
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: %s
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset
	Attributes:
		 ->  code.function.name: String(Owned("DemoClass::test"))
		 ->  code.file.path: String(Owned("/usr/src/myapp/tests/auto/autoinstrument-class.php"))
		 ->  code.line.number: I64(5)
