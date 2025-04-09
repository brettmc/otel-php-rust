--TEST--
Inject context
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\Span;

$parent = Globals::propagator()->extract(['traceparent' => '00-e77388f01a826e2de7afdcd1eefc034e-d6ba64af4fa59b65-01']);
$propagator = Globals::propagator();
$carrier = [];
$propagator->inject($carrier, null, $parent); //TODO fix "name" in phper
var_dump($carrier['traceparent']);
?>
--EXPECT--
string(55) "00-e77388f01a826e2de7afdcd1eefc034e-d6ba64af4fa59b65-01"
