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
//
// Guarded rather than skipping the whole file: the return *values* checked above are the
// point of this test and are worth asserting on every version. Only the declaration needs
// 7.2, because phper-sys gates ZEND_BEGIN_ARG_WITH_RETURN_TYPE_INFO_EX at PHP_VERSION_ID
// >= 70200 and falls back to ZEND_BEGIN_ARG_INFO_EX below it -- so on 7.0/7.1 even a
// builtin return type is never registered and getReturnType() is NULL. (The class-valued
// OBJ variant needs 8.0; see interface-surface.phpt.)
//
// Reports mismatches instead of echoing each signature, so the expected output stays the
// same on every version rather than forking on one.
if (PHP_VERSION_ID >= 70200) {
    foreach ([TracerProvider::class, LoggerProvider::class] as $class) {
        foreach (['forceFlush', 'shutdown'] as $method) {
            // getName(), not string interpolation: casting a ReflectionType to string is
            // deprecated in PHP 7.4 and the notice would land in the middle of the
            // expected output.
            $returnType = (new ReflectionMethod($class, $method))->getReturnType();
            $actual = $returnType === null ? 'none' : $returnType->getName();
            if ($actual !== 'bool') {
                echo $class, '::', $method, '() declares ', $actual, ', expected bool', "\n";
            }
        }
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
bool(true)
bool(true)
bool(true)
bool(true)
