<?php

$globals = new \OpenTelemetry\Globals();
var_dump($globals);
$tracer = $globals->getTracer();
var_dump($tracer);
$tracer->test("foobar");