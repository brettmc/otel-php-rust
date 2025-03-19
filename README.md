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

### Debugging

There's a bunch of logging from the extension and the underlying opentelemetry-rust and dependencies. It's configurable via `.ini`:

`otel.log.level` -> `error` (default), `warn`, `info`, `debug` or `trace`

`otel.log.file` -> `/dev/stderr` (default), or another file/location of your choosing

If you really want to see what's going on, set the log level to `trace` and you'll get a lot of logs.

## SAPI support

### `cli`
http + grpc exporters work. Does not create a root span for this SAPI.

This should cover cli-based PHP runtimes (roadrunner, react, etc), but has only been tested against RoadRunner.

### `cli-server`
http + grpc exporter works. Creates root span on RINIT.

### `apache2handler`
As above

### `cgi-fcgi`
As above

## What works?

* Auto-instrumentation of userland and internal code, via zend_observer API (see `tests/auto/*`)
* Start a span in RINIT, use `traceparent` headers, set HTTP response code in RSHUTDOWN
* TracerProvider created in RINIT (so that child processes have a working instance)
* Spans can be built through a SpanBuilder, some updates made (not all implemented yet), and `end()`ed
* Spans can be `activate()`d, and scope detached
* Spans export to stdout, otlp (grpc + http/protobuf)
* Batch and Simple span processors
* Get SpanContext from a Span
* Access "local root span"

Basic usage:
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

Some more advanced stuff:
```php
$tracer = \OpenTelemetry\API\Globals::tracerProvider()->getTracer('my-tracer');
$root = $tracer->spanBuilder('root')->startSpan();
$scope = $root->activate();

//somewhere else in code
\OpenTelemetry\API\Trace\Span::getLocalRoot()->updateName('updated');

$root->end();
$scope->detach();
```

## What doesn't work or isn't implemented? (todo list)

### Type-hints
* phper only supports basic type-hints (`int`, `object`, etc), not class entries or unions. TODO: add to upstream

## The future

Depending on what we can get to work, or not, this extension could go in a number of directions.

1. An implementation of the opentelemetry-php API, backed by opentelemetry-rust API+SDK
2. Use opentelemetry-rust to support only auto-instrumentation a-la [SkyWalking](https://github.com/apache/skywalking-php/),
[Compass](https://github.com/skpr/compass/).
3. Implement an entirely new API, closer to opentelemetry-rust's (ie, don't try to match opentelemetry-php)
4. Some combination of the above (probably 1+2 or 2+3)
5. Abandon ship, chalk it up to experience
