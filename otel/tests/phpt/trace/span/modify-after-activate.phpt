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
use OpenTelemetry\API\Trace\SpanContext;

$span = Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('root')->startSpan();
$scope = $span->activate();
$span->setStatus('Ok')
     ->setAttribute('foo', 'bar')
     ->setAttributes(['baz' => 'bat', 'num' => 2])
     ->updateName('updated')
     ->recordException(new \Exception('kaboom'))
     //->addLink(SpanContext::create('2b4ef3412d587ce6e7880fb27a316b8c', '7480a670201f6340'))
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