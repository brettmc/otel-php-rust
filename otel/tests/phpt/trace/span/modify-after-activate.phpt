--TEST--
Modify span after activate
--DESCRIPTION--
Span is stored as an SdkSpan or SpanRef, depending on whether it has been activated or not
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\StatusCode;

$span = Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('root')->startSpan();
$scope = $span->activate();
$span->setStatus('Ok')
     ->setAttribute('foo', 'bar')
     ->setAttributes(['baz' => 'bat', 'num' => 2])
     ->updateName('updated')
     ->recordException(new \Exception('kaboom'))
     ->addLink()
     ->addEvent()
     ->end();
 $scope->detach();
?>
--EXPECTF--
Spans
Resource
	 ->  %A
Span #0
	Instrumentation Scope
%A

	Name        : updated
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: 0000000000000000
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Ok
	Attributes:
%A