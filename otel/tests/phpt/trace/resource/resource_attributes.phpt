--TEST--
Detects resource SERVICE_NAME
--EXTENSIONS--
otel
--INI--
otel.cli.enable=1
--ENV--
OTEL_TRACES_EXPORTER=console
OTEL_SPAN_PROCESSOR=simple
OTEL_RESOURCE_ATTRIBUTES=service.name=foo-service,host.name=my-host
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\StatusCode;

$tracer = Globals::tracerProvider()->getTracer('test', '0.1', 'schema.url');
$tracer->spanBuilder('test')->startSpan()->end();
?>
--EXPECTF--
Spans
Resource%A
	 ->  service.name=String(Owned("foo-service"))%A
Span #0
%A