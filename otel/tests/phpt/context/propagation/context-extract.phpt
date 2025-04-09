--TEST--
Extract context
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\API\Globals;

$headers = [
    'foo' => 'bar',
    'traceparent' => '00-e77388f01a826e2de7afdcd1eefc034e-d6ba64af4fa59b65-01',
];

$propagator = Globals::propagator();
?>
--EXPECT--
