--TEST--
Userland implementations of our interfaces are accepted, not reinterpreted as our state
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
// Several arguments are typed as an interface rather than a concrete class, which makes a
// userland implementation reachable inside the extension -- where `as_state_obj`
// reinterprets an object's memory as our Rust state, and is only sound for a class this
// extension defined. A foreign object reaching one of those casts is a crash, not an
// exception, so each one checks the class first. This drives every such argument with a
// foreign object.
//
// A crash shows up as a *dead* test rather than a failed assertion, which is why this file
// asserts on values throughout: reaching "DONE" is itself the thing being proved.
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Trace\SpanBuilderInterface;
use OpenTelemetry\API\Trace\SpanContextInterface;
use OpenTelemetry\API\Trace\SpanInterface;
use OpenTelemetry\API\Trace\TraceStateInterface;
use OpenTelemetry\Context\Context;
use OpenTelemetry\Context\ContextInterface;
use OpenTelemetry\Context\ContextKeyInterface;
use OpenTelemetry\Context\ContextStorageScopeInterface;
use OpenTelemetry\Context\ImplicitContextKeyedInterface;
use OpenTelemetry\Context\ScopeInterface;

final class UserlandContext implements ContextInterface
{
    public static function createKey(string $key): ContextKeyInterface { return Context::createKey($key); }
    public static function getCurrent(): ContextInterface { return Context::getCurrent(); }
    public function activate(): ScopeInterface { throw new LogicException('not needed'); }
    public function storeInContext(ContextInterface $context): ContextInterface { return $context; }
    public function with(ContextKeyInterface $key, $value): ContextInterface { return $this; }
    public function withContextValue(ImplicitContextKeyedInterface $value): ContextInterface { return $this; }
    public function get(ContextKeyInterface $key) { return null; }
}

final class UserlandSpanContext implements SpanContextInterface
{
    public static function create(string $traceId, string $spanId, int $traceFlags = 0, ?TraceStateInterface $traceState = null): SpanContextInterface { return new self(); }
    public static function createFromRemoteParent(string $traceId, string $spanId, int $traceFlags = 0, ?TraceStateInterface $traceState = null): SpanContextInterface { return new self(); }
    public static function getInvalid(): SpanContextInterface { return new self(); }
    public function getTraceId(): string { return 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'; }
    public function getSpanId(): string { return 'bbbbbbbbbbbbbbbb'; }
    public function getTraceFlags(): int { return 1; }
    public function isValid(): bool { return true; }
    public function isRemote(): bool { return false; }
    public function isSampled(): bool { return true; }
}

// `Storage::attach()` is typed ContextInterface, so a userland context reaches the cast
// that reads our `Arc<Context>`. A foreign object has none, which resolves to the current
// context rather than to garbage.
$scope = Context::storage()->attach(new UserlandContext());
var_dump($scope instanceof ContextStorageScopeInterface);
var_dump($scope->context() instanceof Context);
var_dump($scope->detach());

$tracer = Globals::tracerProvider()->getTracer('t');

// `SpanBuilder::setParent()` is typed ?ContextInterface for the same reason.
$span = $tracer->spanBuilder('with-userland-parent')->setParent(new UserlandContext())->startSpan();
var_dump($span->getContext()->isValid());
$span->end();

// `addLink()` on both the builder and the span names SpanContextInterface, as upstream does.
// A userland span context carries everything needed, so it is rebuilt through the interface
// accessors rather than refused.
$builder = $tracer->spanBuilder('with-userland-link')->addLink(new UserlandSpanContext());
var_dump($builder instanceof SpanBuilderInterface);
$linked = $builder->startSpan();
var_dump($linked instanceof SpanInterface);
var_dump($linked->addLink(new UserlandSpanContext()) instanceof SpanInterface);
$linked->end();

echo "DONE\n";
?>
--EXPECT--
bool(true)
bool(true)
int(0)
bool(true)
bool(true)
bool(true)
bool(true)
DONE
