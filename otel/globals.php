<?php

use OpenTelemetry\Globals;

$tracer = Globals::getTracer();
var_dump($tracer);
$tracer->test("foobar");