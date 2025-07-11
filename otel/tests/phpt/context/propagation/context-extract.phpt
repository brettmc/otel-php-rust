--TEST--
Extract context
--EXTENSIONS--
otel
--INI--
otel.cli.enable=1
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\Span;

$headers = [
    'foo' => 'bar',
    'traceparent' => '00-e77388f01a826e2de7afdcd1eefc034e-d6ba64af4fa59b65-01',
];

$propagator = Globals::propagator();
$ctx = $propagator->extract($headers);
$span = Span::fromContext($ctx);
$spanContext = $span->getContext();

var_dump($spanContext->getTraceId());
var_dump($spanContext->getSpanId());
?>
--EXPECT--
string(32) "e77388f01a826e2de7afdcd1eefc034e"
string(16) "d6ba64af4fa59b65"
