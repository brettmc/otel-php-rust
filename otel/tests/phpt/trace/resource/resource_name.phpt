--TEST--
Detects resource SERVICE_NAME
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
OTEL_SPAN_PROCESSOR=simple
OTEL_SERVICE_NAME=service_one
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\StatusCode;

$tracer = Globals::tracerProvider()->getTracer('test', '0.1', 'schema.url');
$tracer->spanBuilder('test')->startSpan()->end();
/* NB that resource attributes are displayed in undefined order, so
   only assert on service.name */
?>
--EXPECTF--
Spans
Resource%A
	 ->  service.name=String(Owned("service_one"))%A
Span #0
%A