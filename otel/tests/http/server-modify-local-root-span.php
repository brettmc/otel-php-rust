<?php
use OpenTelemetry\API\Trace\LocalRootSpan;

header('Content-Type: text/plain');
http_response_code(201);

LocalRootSpan::current()->updateName("I was updated via LocalRootSpan");