--TEST--
Inject the current context
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=1
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\Context\Context;

$span = Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('root')->startSpan();
$scope = $span->activate();

// Context::getCurrent() reports a context_id of 0, so there is no stored instance to look
// up; the propagator has to fall back to the current context rather than failing.
$carrier = [];
Globals::propagator()->inject($carrier, null, Context::getCurrent());
var_dump(isset($carrier['traceparent']));
var_dump($carrier['traceparent'] === '00-' . $span->getContext()->getTraceId() . '-' . $span->getContext()->getSpanId() . '-01');

$scope->detach();
$span->end();
?>
--EXPECT--
bool(true)
bool(true)
