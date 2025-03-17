--TEST--
Detects resource SERVICE_NAME
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
OTEL_RESOURCE_ATTRIBUTES=service.name=foo-service,host.name=my-host
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\StatusCode;

$tracer = Globals::tracerProvider()->getTracer('test');
$tracer->spanBuilder('test')->startSpan()->end();
?>
--EXPECTF--
Spans
Resource
%A
	 ->  service.name=String(Owned("foo-service"))
%A
Span #0
%A