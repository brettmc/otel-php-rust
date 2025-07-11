--TEST--
Leave a dangling span
--SKIPIF--
<?php
if (PHP_VERSION_ID < 80000) {
    die("skip requires RSHUTDOWN output only available in PHP 8.0+");
}
?>
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=none
--INI--
otel.log.level="trace"
;use stderr to avoid interfering with SKIPIF output
otel.log.file="/dev/stderr"
otel.cli.enable=1
--FILE--
<?php
use OpenTelemetry\API\Globals;

$span = Globals::tracerProvider()
    ->getTracer('my_tracer', '0.1', 'schema.url')
    ->spanBuilder('dangling-span')
    ->startSpan();
$scope = $span->activate();
// Do not end the span, leaving it dangling (it will close on shutdown, note difference from opentelemetry-php)
?>
--EXPECTF--
%A
[%s] [DEBUG] [pid=%d] [ThreadId(%d)] OpenTelemetry::RSHUTDOWN
%A
[%s] [WARN] [pid=%d] [ThreadId(%d)] otel::request: event src/request.rs:%d message=RSHUTDOWN::context still stored: [%d]
%A
