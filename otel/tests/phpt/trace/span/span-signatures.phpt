--TEST--
Span declares every argument it reads, and NonRecordingSpan matches it
--SKIPIF--
<?php
if (PHP_VERSION_ID < 80000) {
    die("skip class type hints are not registered before PHP 8.0");
}
?>
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=1
--ENV--
OTEL_TRACES_EXPORTER=none
--FILE--
<?php
// `declared-types-exist.phpt` checks that every declared type *resolves*; it cannot see an
// argument that was never declared at all. This pins the signatures themselves, so an
// argument the handler reads but does not declare shows up as a diff here rather than as
// an abort at runtime.
//
// Span and NonRecordingSpan are compared against each other as well: both implement
// SpanInterface, and userland holding one does not know which it has. An arity that
// differs between them makes the same call legal against one and a TypeError against the
// other -- and NonRecordingSpan's methods are all no-ops, so nothing else would catch it.

function signature(string $class, string $method): string {
    $m = new ReflectionMethod($class, $method);
    $params = [];
    foreach ($m->getParameters() as $p) {
        $params[] = sprintf(
            '%s$%s%s',
            $p->hasType() ? $p->getType() . ' ' : '',
            $p->getName(),
            $p->isDefaultValueAvailable()
                ? ' = ' . str_replace(["\n", ' '], '', var_export($p->getDefaultValue(), true))
                : ''
        );
    }

    return sprintf('(%s)', implode(', ', $params));
}

$methods = [
    'setStatus', 'setAttribute', 'setAttributes', 'updateName',
    'recordException', 'addLink', 'addEvent', 'storeInContext', 'fromContext',
    // `end` carries upstream's optional end timestamp, and `isRecording` is the question
    // every instrumentation asks before doing work -- both have to be reachable through a
    // SpanInterface, so both are pinned here.
    'end', 'isRecording', 'getContext', 'getCurrent', 'activate',
];

foreach ($methods as $method) {
    $span = signature('OpenTelemetry\API\Trace\Span', $method);
    $nonRecording = signature('OpenTelemetry\API\Trace\NonRecordingSpan', $method);
    printf("%-16s %s%s\n", $method, $span, $span === $nonRecording ? '' : " != NonRecordingSpan $nonRecording");
}
?>
--EXPECT--
setStatus        (string $code, ?string $description = NULL)
setAttribute     (string $key, mixed $value)
setAttributes    (iterable $attributes)
updateName       (string $name)
recordException  (Throwable $exception, iterable $attributes = array())
addLink          (OpenTelemetry\API\Trace\SpanContextInterface $context, iterable $attributes = array())
addEvent         (string $name, iterable $attributes = array(), ?int $timestamp = NULL)
storeInContext   (OpenTelemetry\Context\ContextInterface $context)
fromContext      (OpenTelemetry\Context\ContextInterface $context)
end              (?int $endEpochNanos = NULL)
isRecording      ()
getContext       ()
getCurrent       ()
activate         ()
