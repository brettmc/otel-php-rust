--TEST--
OTEL_SDK_DISABLED=true in env, =false in .env file
--EXTENSIONS--
otel
--INI--
otel.env.dotenv.enabled=On
otel.cli.enabled=On
otel.log.level=error
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SDK_DISABLED=true
OTEL_SPAN_PROCESSOR=simple
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

$tracer = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url');

$tracer->spanBuilder('root')->startSpan()->end();

assert(Memory::count() === 1);
var_dump(getenv('OTEL_SDK_DISABLED')); //getenv because updating env from RINIT doesn't update $_SERVER
?>
--EXPECTF--
string(5) "false"