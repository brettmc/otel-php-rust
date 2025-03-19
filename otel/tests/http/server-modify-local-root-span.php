<?php
use OpenTelemetry\API\Trace\Span;

header('Content-Type: text/plain');
http_response_code(201);

Span::getLocalRoot()->updateName("I was updated via LocalRootSpan");