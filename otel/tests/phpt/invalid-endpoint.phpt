--TEST--
Invalid OTLP endpoint disables export without killing the process
--DESCRIPTION--
A malformed OTEL_EXPORTER_OTLP_ENDPOINT makes the OTLP exporter fail to build.
That must degrade to no-export and log the reason, rather than panicking and
aborting the PHP process.
--EXTENSIONS--
otel
--INI--
otel.log.level="error"
otel.log.file="/dev/stdout"
otel.cli.enabled=1
--ENV--
OTEL_EXPORTER_OTLP_ENDPOINT=not a url
--FILE--
<?php
use OpenTelemetry\API\Globals;

Globals::tracerProvider()
    ->getTracer('my_tracer', '0.1', 'schema.url')
    ->spanBuilder('root')
    ->startSpan()
    ->end();
Globals::tracerProvider()->forceFlush();
echo "still alive\n";
?>
--EXPECTF--
[%s] [ERROR] [pid=%d] [ThreadId(%d)] otel::trace::tracer_provider: event src/trace/tracer_provider.rs:%d message=Failed to create OTLP trace exporter, tracing is disabled: %s
%A
still alive
