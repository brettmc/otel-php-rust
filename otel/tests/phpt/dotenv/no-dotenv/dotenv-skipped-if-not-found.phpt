--TEST--
dotenv skipped if not found
--EXTENSIONS--
otel
--INI--
otel.cli.enable=On
otel.dotenv.per_request=On
otel.log.level=debug
--ENV--
OTEL_TRACES_EXPORTER=console
OTEL_SPAN_PROCESSOR=simple
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\SpanExporter\Memory;
Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('root')->startSpan()->end();
?>
--EXPECTF--
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] OpenTelemetry::RINIT::Loading .env file%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] OpenTelemetry::RINIT::No .env file found in "/usr/src/myapp/tests/phpt/dotenv/no-dotenv"
%A
Spans
Resource%A
	 ->  service.name=String(Static("unknown_service"))
%A