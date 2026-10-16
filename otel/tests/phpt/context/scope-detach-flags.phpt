--TEST--
ScopeInterface's detach flags match PHP's own integer width
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=none
OTEL_LOGS_EXPORTER=none
OTEL_METRICS_EXPORTER=none
--INI--
otel.cli.enabled=1
--FILE--
<?php
use OpenTelemetry\Context\ScopeInterface;

// Upstream defines these as `1 << (PHP_INT_SIZE << 3) - N`, i.e. relative to the top of
// PHP's integer. Comparing against that expression rather than against literals means this
// test is also correct on a 32-bit build, where the values are different numbers but the
// constants must still agree with the engine that interprets them.
var_dump(ScopeInterface::DETACHED === 1 << (PHP_INT_SIZE << 3) - 1);
var_dump(ScopeInterface::INACTIVE === 1 << (PHP_INT_SIZE << 3) - 2);
var_dump(ScopeInterface::MISMATCH === 1 << (PHP_INT_SIZE << 3) - 3);

// Each flag occupies its own bit, so userland's `$flags & ScopeInterface::DETACHED` cannot
// be confused by another condition's value.
var_dump(ScopeInterface::DETACHED & ScopeInterface::MISMATCH);
var_dump(ScopeInterface::DETACHED & ScopeInterface::INACTIVE);
var_dump(ScopeInterface::INACTIVE & ScopeInterface::MISMATCH);
?>
--EXPECT--
bool(true)
bool(true)
bool(true)
int(0)
int(0)
int(0)
