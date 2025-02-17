# OpenTelemetry + Rust

## Intro

This is a prototype PHP extension, using [phper](https://github.com/phper-framework/phper)
to expose [opentelemetry-rust](https://opentelemetry.io/docs/languages/rust/) via PHP
classes.

The initial idea was to implement the PHP API in an extension, which could be a
drop-in replacement for the core of opentelemetry-php. Since 3rd parties are
strongly encouraged to only depend on the API, this should be all they need.

## Development

Using docker + compose, `Dockerfile` provides PHP 8.4 + rust environment to build and
test in. I've added `Makefile`s to make things easier, and written some tests using PHP's
[phpt](https://qa.php.net/phpt_details.php).

Quick-start:

* `git clone <this repo>`
* `make build`
* `make bash`

From this bash shell, there is another Makefile to build the extension and run the tests.
Tests are organised as:

- `auto` - auto-instrumentation
- `otlp` - otlp (`http/protobuf` + `grpc`) exporting to a local collector
- `phpt` - test the API via PHP code

For the `otlp` tests, be sure to `docker compose up -d collector` first.

## What works?

* Auto-instrumentation of userland code (see `tests/auto/*`)
* TracerProvider globally registered in MINIT, and shutdown on MSHUTDOWN
* Spans can be built through a SpanBuilder, some updated made (not all implemented yet), and `end()`ed
* Spans export to stdout, otlp
* Get SpanContext from a Span

```php
$provider = \OpenTelemetry\API\Globals::tracerProvider();
$tracer = $provider->getTracer('name', '0.1' /*other params*/);
$span = $tracer
    ->spanBuilder('test-span')
    ->setAttribute('key', 'value');
$span->updateName('updated');
var_dump($span->getContext()->getTraceId());
$span
    ->setStatus('Ok')
    ->end();
```

## What doesn't work? (todo list)

The biggest thing that doesn't work is being able to `activate()` a span (or a context), so that
later spans are parented to the active span. I can't work out how to store the returned `guard`
so that it can be de-referenced later.

### SpanBuilder
* doesn't keep a reference to the tracer, and instead fetches a new tracer each time (losing any InstrumentationScope)

### Span
* `activate()` only keeps the span active until the returned `guard` immediately goes out of scope. Need to stash the 
guard in another class, eg `Scope` to allow `Scope->detach()`
* `getCurrent()` can fetch the current span as a `SpanRef`, but doesn't do anything useful with it
* `setAttributes`, `addEvent`, `addLink`, `recordException` are no-ops

### Scope
* not implemented

### StatusCode
* not implemented. PR accepted in `phper` to allow adding consts to classes & interfaces to enable this.

### Auto-instrumented internal functions
Seems to be an issue with `zend_execute_internal` not being initialized at MINIT or RINIT.

## The future

Depending on what we can get to work, or not, this extension could go in a number of directions.

1. An implementation of the opentelemetry-php API, backed by opentelemetry-rust API+SDK
2. Do not expose any classes, and use opentelemetry-rust to support only auto-instrumentation
a-la [SkyWalking](https://github.com/apache/skywalking-php/), [Compass](https://github.com/skpr/compass/).
3. Implement an entirely new API, closer to opentelemetry-rust's (ie, don't try to match opentelemetry-php)
4. Some combination of the above (probably 1+2 or 2+3)
5. Abandon ship, chalk it up to experience
