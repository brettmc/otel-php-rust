--TEST--
dotenv support enabled
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=On
otel.env.dotenv.enabled=On
otel.log.level=debug
--ENV--
OTEL_TRACES_EXPORTER=console
OTEL_SPAN_PROCESSOR=simple
OTEL_SERVICE_NAME=do-not-use
OTEL_RESOURCE_ATTRIBUTES=service.namespace=do-not-use
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\SpanExporter\Memory;
Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('root')->startSpan()->end();
?>
--EXPECTF--
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] otel::trace::tracer_provider: event src/trace/tracer_provider.rs:%d message=creating tracer provider for key (%d, "from-dotenv:service.namespace=my-dotenv-service,service.version=0.1.0")
%A
Spans
Resource%A
	 ->  service.name=String(Owned("from-dotenv"))%A
