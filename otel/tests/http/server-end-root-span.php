<?php
use OpenTelemetry\API\Trace\Span;

header('Content-Type: text/plain');
http_response_code(201);

Span::getCurrent()->end();
//var_dump($span->getContext()->isValid());
