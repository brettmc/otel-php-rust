--TEST--
Create a span with all features
--EXTENSIONS--
otel
--FILE--
<?php
use OpenTelemetry\Globals;
use OpenTelemetry\API\Trace\StatusCode;

$span = Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('root')->startSpan();
$span->setStatus('Ok')
     ->setAttribute('foo', 'bar')
     ->setAttributes(['baz' => 'bat', 'num' => 2])
     ->updateName('updated')
     ->recordException(new \Exception('kaboom'))
     ->addLink()
     ->addEvent()
     ->end();
?>
--EXPECTF--
Spans
Resource
	 ->  %A
Span #0
	Instrumentation Scope
		Name         : "change-me"
		Version  : "0.1"
		SchemaUrl: "http://my.schema.url"

	Name        : updated
	TraceId     : %s
	SpanId      : %s
	ParentSpanId: 0000000000000000
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Ok
	Attributes:
		 ->  foo: String(Owned("bar"))