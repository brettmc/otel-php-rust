<?php

use OpenTelemetry\API\Globals;

echo "starting...\n";
$globals = new \OpenTelemetry\Globals();
var_dump($globals->getFoo());
var_dump($globals->otel());
echo "finished...\n";
