--TEST--
Every interface the extension registers declares upstream's method names and arity
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=1
--ENV--
OTEL_TRACES_EXPORTER=none
OTEL_LOGS_EXPORTER=none
OTEL_METRICS_EXPORTER=none
--FILE--
<?php
// `declared-types-exist.phpt` checks that declared types *resolve*; it cannot see a method
// that was never declared. An interface registers happily with no methods at all, and
// userland holding it then cannot call methods the concrete class implements perfectly
// well. This test covers that gap: it asserts the methods are there.
//
// Each expectation below is upstream's own signature (open-telemetry/opentelemetry-php),
// pinned as name => [requiredParams, totalParams]. Arity, not full types, because phper
// cannot express some of upstream's types (unions especially) so those legitimately differ --
// a missing *method* never does.
$expected = [
    'OpenTelemetry\Context\ContextInterface' => [
        'createKey' => [1, 1],
        'getCurrent' => [0, 0],
        'activate' => [0, 0],
        'with' => [2, 2],
        'withContextValue' => [1, 1],
        'get' => [1, 1],
    ],
    // A pure marker upstream: no methods, no constants. It exists so `with()`/`get()` can
    // name a key type. `ContextKey::name()` is on the class, not the interface.
    'OpenTelemetry\Context\ContextKeyInterface' => [],
    'OpenTelemetry\Context\ImplicitContextKeyedInterface' => [
        'activate' => [0, 0],
        'storeInContext' => [1, 1],
    ],
    'OpenTelemetry\Context\ScopeInterface' => [
        // `context()` deliberately absent: upstream puts it on ContextStorageScopeInterface.
        'detach' => [0, 0],
    ],
    'OpenTelemetry\Context\ContextStorageScopeInterface' => [
        'context' => [0, 0],
        'detach' => [0, 0],
        'offsetExists' => [1, 1],
        'offsetGet' => [1, 1],
        'offsetSet' => [2, 2],
        'offsetUnset' => [1, 1],
    ],
    'OpenTelemetry\Context\ContextStorageInterface' => [
        'scope' => [0, 0],
        'current' => [0, 0],
        'attach' => [1, 1],
    ],
    'OpenTelemetry\Context\Propagation\TextMapPropagatorInterface' => [
        'inject' => [1, 3],
        'extract' => [1, 3],
    ],
    'OpenTelemetry\Context\Propagation\PropagationGetterInterface' => [
        'keys' => [1, 1],
        'get' => [2, 2],
    ],
    'OpenTelemetry\Context\Propagation\PropagationSetterInterface' => [
        'set' => [3, 3],
    ],
    'OpenTelemetry\API\Trace\SpanContextInterface' => [
        'create' => [2, 3],
        'createFromRemoteParent' => [2, 3],
        'getInvalid' => [0, 0],
        'getTraceId' => [0, 0],
        'getSpanId' => [0, 0],
        'getTraceFlags' => [0, 0],
        'isValid' => [0, 0],
        'isRemote' => [0, 0],
        'isSampled' => [0, 0],
    ],
    'OpenTelemetry\API\Trace\TracerInterface' => [
        'spanBuilder' => [1, 1],
    ],
    'OpenTelemetry\API\Trace\SpanBuilderInterface' => [
        'setParent' => [1, 1],
        'addLink' => [1, 2],
        'setAttribute' => [2, 2],
        'setAttributes' => [1, 1],
        'setStartTimestamp' => [1, 1],
        'setSpanKind' => [1, 1],
        'startSpan' => [0, 0],
    ],
    'OpenTelemetry\API\Trace\SpanInterface' => [
        'getCurrent' => [0, 0],
        'fromContext' => [1, 1],
        'getContext' => [0, 0],
        'isRecording' => [0, 0],
        'setAttribute' => [2, 2],
        'setAttributes' => [1, 1],
        'addEvent' => [1, 3],
        'addLink' => [1, 2],
        'recordException' => [1, 2],
        'updateName' => [1, 1],
        'setStatus' => [1, 2],
        'end' => [0, 1],
        // Inherited from ImplicitContextKeyedInterface, as upstream inherits them.
        'activate' => [0, 0],
        'storeInContext' => [1, 1],
    ],
];

$errors = [];
foreach ($expected as $interface => $methods) {
    if (!interface_exists($interface)) {
        $errors[] = "$interface does not exist";
        continue;
    }
    $reflection = new ReflectionClass($interface);
    foreach ($methods as $method => [$required, $total]) {
        if (!$reflection->hasMethod($method)) {
            $errors[] = "$interface::$method() is not declared";
            continue;
        }
        $m = $reflection->getMethod($method);
        if ($m->getNumberOfRequiredParameters() !== $required || $m->getNumberOfParameters() !== $total) {
            $errors[] = sprintf(
                '%s::%s() takes %d/%d parameters, expected %d/%d',
                $interface,
                $method,
                $m->getNumberOfRequiredParameters(),
                $m->getNumberOfParameters(),
                $required,
                $total,
            );
        }
    }
}

// `context()` moved off ScopeInterface; assert the move, not just the arrival, or the
// over-promise could silently come back.
if (method_exists('OpenTelemetry\Context\ScopeInterface', 'context')) {
    $errors[] = 'ScopeInterface::context() should have moved to ContextStorageScopeInterface';
}

// Return types, checked separately because the arity loop above cannot see one: a signature
// can keep its parameters and lose its return type, and these three are the ones that make a
// fluent chain checkable past the first arrow.
$returns = [
    ['OpenTelemetry\API\Trace\TracerInterface', 'spanBuilder', 'OpenTelemetry\API\Trace\SpanBuilderInterface'],
    ['OpenTelemetry\API\Trace\Tracer', 'spanBuilder', 'OpenTelemetry\API\Trace\SpanBuilderInterface'],
    // Upstream's is the intersection `ContextStorageInterface&ExecutionContextAwareInterface`;
    // phper cannot express an intersection, so we declare the half that is true.
    ['OpenTelemetry\Context\Context', 'storage', 'OpenTelemetry\Context\ContextStorageInterface'],
];
foreach ($returns as [$class, $method, $expectedReturn]) {
    $actual = (string) (new ReflectionMethod($class, $method))->getReturnType();
    if ($actual !== $expectedReturn) {
        $errors[] = "$class::$method() returns '$actual', expected '$expectedReturn'";
    }
}

// Every concrete class must still satisfy the interfaces above. PHP enforces this at
// registration, so a divergence is a fatal module-load error rather than something this
// test could observe -- what it can observe is that the implements relationships are the
// ones intended.
$implements = [
    'OpenTelemetry\Context\Context' => 'OpenTelemetry\Context\ContextInterface',
    'OpenTelemetry\Context\ContextKey' => 'OpenTelemetry\Context\ContextKeyInterface',
    'OpenTelemetry\Context\Scope' => 'OpenTelemetry\Context\ContextStorageScopeInterface',
    'OpenTelemetry\Context\Storage' => 'OpenTelemetry\Context\ContextStorageInterface',
    'OpenTelemetry\API\Trace\Span' => 'OpenTelemetry\API\Trace\SpanInterface',
    'OpenTelemetry\API\Trace\NonRecordingSpan' => 'OpenTelemetry\API\Trace\SpanInterface',
    'OpenTelemetry\API\Trace\SpanContext' => 'OpenTelemetry\API\Trace\SpanContextInterface',
    // Upstream's concrete builder is `SDK\Trace\SpanBuilder`; ours is under `API\Trace`.
    // Pinned as-is rather than silently: the interface is what userland types against, and
    // it is in the right namespace.
    'OpenTelemetry\API\Trace\SpanBuilder' => 'OpenTelemetry\API\Trace\SpanBuilderInterface',
    'OpenTelemetry\API\Trace\Propagation\TraceContextPropagator' => 'OpenTelemetry\Context\Propagation\TextMapPropagatorInterface',
];
foreach ($implements as $class => $interface) {
    if (!class_exists($class)) {
        $errors[] = "$class does not exist";
    } elseif (!is_subclass_of($class, $interface)) {
        $errors[] = "$class does not implement $interface";
    }
}

if ($errors) {
    echo implode("\n", $errors), "\n";
}
echo "checked ", count($expected), " interfaces\n";
?>
--EXPECT--
checked 13 interfaces
