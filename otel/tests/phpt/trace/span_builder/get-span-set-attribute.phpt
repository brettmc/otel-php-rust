--TEST--
Call setAttribute multiple times on SpanBuilder
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\StatusCode;

$builder = Globals::tracerProvider()->getTracer("my_tracer")->spanBuilder('root');
$builder->setAttribute('foo', 'bar')
    ->setAttribute('baz', 'bat');
$span = $builder->startSpan();
$span->end();
?>
--EXPECTF--
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "php_rust"

	Name        : root
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: 0000000000000000
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset
	Attributes:
		 ->  foo: String(Owned("bar"))
		 ->  baz: String(Owned("bat"))
