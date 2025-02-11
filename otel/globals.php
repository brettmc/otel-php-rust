<?php

use OpenTelemetry\Globals;

$provider = Globals::tracerProvider();
var_dump($provider);
$tracer = $provider->getTracer("my_tracer");
var_dump($tracer);
$tracer->test("foobar");