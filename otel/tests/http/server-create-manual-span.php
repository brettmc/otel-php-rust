<?php
use OpenTelemetry\API\Globals;
header('Content-Type: text/plain');
http_response_code(201);
$provider = Globals::tracerProvider();
$span = $provider
    ->getTracer('my_tracer')
    ->spanBuilder('manual')
    ->startSpan();
var_dump($span->getContext()->getTraceId());
var_dump($span->getContext()->getSpanId());
$span->end();
//sleep(5);
