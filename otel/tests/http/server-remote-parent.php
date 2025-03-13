<?php
use OpenTelemetry\API\Trace\Span;
header('Content-Type: text/plain');
$span = Span::getCurrent();
var_dump($span->getContext()->getTraceId());
var_dump($span->getContext()->getSpanId());
var_dump($span->getContext()->isRemote()); //this span is not remote, but its parent is...
$span->end();
