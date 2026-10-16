--TEST--
forceFlush and shutdown report success instead of returning NULL
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_LOGS_EXPORTER=none
OTEL_METRICS_EXPORTER=none
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.cli.enabled=1
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Logs\LoggerProvider;
use OpenTelemetry\API\Trace\TracerProvider;

$tracerProvider = Globals::tracerProvider();
$loggerProvider = Globals::loggerProvider();

// var_dump, not assert: the failure mode worth catching is a NULL return, and both
// `assert($x)` and a truthiness check would report that identically to a flush that
// genuinely failed. The point of this test is that the value is a real bool.
var_dump($tracerProvider->forceFlush());
var_dump($loggerProvider->forceFlush());

// A declared return type on an internal function is not enforced by the engine, so a
// handler returning Rust `()` would satisfy `: bool` and hand userland NULL anyway.
// Checking the declaration as well as the value catches the inverse regression: a correct
// value with the declaration dropped.
foreach ([TracerProvider::class, LoggerProvider::class] as $class) {
    foreach (['forceFlush', 'shutdown'] as $method) {
        // getName(), not string interpolation: casting a ReflectionType to string is
        // deprecated in PHP 7.4 and the notice would land in the middle of the expected
        // output. `bool` is a builtin type hint, which phper-sys registers on 7.x as well
        // as 8.x, so there is a real type to read on every version we support.
        $returnType = (new ReflectionMethod($class, $method))->getReturnType();
        echo $class, '::', $method, '(): ', $returnType->getName(), "\n";
    }
}

$tracer = $tracerProvider->getTracer('flush_test');
$tracer->spanBuilder('flushed')->startSpan()->end();
var_dump($tracerProvider->forceFlush());

// shutdown() lets userland release a provider before the request ends, rather than leaving
// it to the teardown MSHUTDOWN runs.
var_dump($tracerProvider->shutdown());
var_dump($loggerProvider->shutdown());

// Shutting down an already-shut-down provider has nothing left to fail at: the registry is
// empty, so the postcondition the caller asked for already holds.
var_dump($tracerProvider->shutdown());
?>
--EXPECT--
bool(true)
bool(true)
OpenTelemetry\API\Trace\TracerProvider::forceFlush(): bool
OpenTelemetry\API\Trace\TracerProvider::shutdown(): bool
OpenTelemetry\API\Logs\LoggerProvider::forceFlush(): bool
OpenTelemetry\API\Logs\LoggerProvider::shutdown(): bool
bool(true)
bool(true)
bool(true)
bool(true)
