# OpenTelemetry + Rust

## Intro

This is a prototype PHP extension, using [phper](https://github.com/phper-framework/phper)
to expose [opentelemetry-rust](https://opentelemetry.io/docs/languages/rust/) via PHP
classes and implement auto-instrumentation.

The initial idea was to implement the PHP API in an extension, which could be a
drop-in replacement for the core of opentelemetry-php. Since 3rd parties are
strongly encouraged to only depend on the API, this should be all they need.

## Installation

### PIE (PHP Installer for Extensions)

Requires `llvm-dev`, `libclang-dev`, rust compiler and cargo.

`php pie.phar install brettmc/otel-php-rust:dev-main`

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

- `test` - all tests
- `test-auto` - auto-instrumentation
- `test-otlp` - otlp (`http/protobuf` + `grpc`) exporting to a local collector
- `test-phpt` - test the API via PHP code

For the `otlp` tests, be sure to `docker compose up -d collector` first.

## SAPI support

### `cli`
http + grpc exporters work. Does not create a root span for this SAPI.

This should cover cli-based PHP runtimes (roadrunner, react, etc), but has only been tested against RoadRunner.

### `cli-server`
http + grpc exporter works. Creates root span on RINIT.

### `apache2handler`
http/protobuf + grpc exporters work.

### `cgi-fcgi`
Same as apache2handler

## What works?

* Auto-instrumentation of userland and internal code, via zend_observer API (see `tests/auto/*`)
* Start a span in RINIT, use `traceparent` headers, set HTTP response code in RSHUTDOWN
* TracerProvider created in RINIT (so that child processes have a working instance)
* Spans can be built through a SpanBuilder, some updates made (not all implemented yet), and `end()`ed
* Spans can be `activate()`d, and scope detached
* Spans export to stdout, otlp (grpc + http/protobuf)
* Batch and Simple span processors
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

## What doesn't work or isn't implemented? (todo list)

### Tracers

Tracers are re-fetched all over the shop from tracer_provider.rs

### SpanBuilder
* doesn't keep a reference to the tracer, and instead fetches a new tracer each time (losing any InstrumentationScope)

### StatusCode
* not implemented. PR accepted in `phper` to allow adding consts to classes & interfaces to enable this.

## The future

Depending on what we can get to work, or not, this extension could go in a number of directions.

1. An implementation of the opentelemetry-php API, backed by opentelemetry-rust API+SDK
2. Do not expose any classes, and use opentelemetry-rust to support only auto-instrumentation
a-la [SkyWalking](https://github.com/apache/skywalking-php/), [Compass](https://github.com/skpr/compass/).
3. Implement an entirely new API, closer to opentelemetry-rust's (ie, don't try to match opentelemetry-php)
4. Some combination of the above (probably 1+2 or 2+3)
5. Abandon ship, chalk it up to experience
