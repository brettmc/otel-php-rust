<?php

use OpenTelemetry\Globals;
use OpenTelemetry\API\Trace\StatusCode;

$provider = Globals::tracerProvider();
var_dump($provider);
$tracer = $provider->getTracer("my_tracer");
var_dump($tracer);
// $tracer->test("foobar");
$builder = $tracer->spanBuilder('root');
var_dump($builder);
$span = $builder->startSpan();
var_dump($span);
$span->setStatus(StatusCode::STATUS_OK)->end();