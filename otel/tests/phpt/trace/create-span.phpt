--TEST--
Create a span
--FILE--
<?php
use OpenTelemetry\Globals;
use OpenTelemetry\API\Trace\StatusCode;

$provider = Globals::tracerProvider();
var_dump($provider);
$tracer = $provider->getTracer("my_tracer");
var_dump($tracer);
$builder = $tracer->spanBuilder('root');
var_dump($builder);
$span = $builder->startSpan();
var_dump($span);
$span->setStatus('Ok')->end();
?>
--EXPECTF--
object(OpenTelemetry\API\Trace\TracerProvider)#1 (0) {
}
object(OpenTelemetry\API\Trace\Tracer)#2 (0) {
}
object(OpenTelemetry\API\Trace\SpanBuilder)#3 (0) {
}
object(OpenTelemetry\API\Trace\Span)#4 (0) {
}
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "change-me"

	Name        : root
	TraceId     : %s
	SpanId      : %s
	ParentSpanId: 0000000000000000
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Ok