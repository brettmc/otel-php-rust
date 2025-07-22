# OpenTelemetry + Rust

## Intro

This is a prototype PHP extension, using [phper](https://github.com/phper-framework/phper)
to expose [opentelemetry-rust](https://opentelemetry.io/docs/languages/rust/) via PHP
classes and implement auto-instrumentation.

The initial idea was to implement the PHP API in an extension, which could be a
drop-in replacement for the core of opentelemetry-php. Since 3rd parties are
strongly encouraged to only depend on the API, this should be all they need.

## Supported PHP versions

### PHP 8.x

Works, and can do some auto-instrumentation via the zend_observer API.

### PHP 7.x

Works (runs and exports traces via OTLP). Can do auto-instrumentation of userland code via `zend_execute_ex`.
Note that writing to stdout/stderr during MSHUTDOWN doesn't seem to work in PHP 7.x, so console exporting and
log writing do not work for this stage. Logging to a file does work, and exporting via OTLP (http/protobuf + grpc) does work.

## Installation

### PIE (PHP Installer for Extensions)

Requires `llvm-dev`, `libclang-dev`, rust compiler and cargo.

`php pie.phar install brettmc/otel-php-rust:<version>`

## Development

Using docker + compose, `Dockerfile` provides PHP 7.0-8.4 + rust environment to build and
test in. I've added `Makefile`s to make things easier, and written some tests using PHP's
[phpt](https://qa.php.net/phpt_details.php).

Quick-start:

* `git clone <this repo>`
* `PHP_VERSION=x.y make build`
* `PHP_VERSION=x.y make bash`

From this bash shell, there is another Makefile to build the extension and run the tests.
Tests are organised as:

- `make clean` - clean up. Make sure you do this when switching PHP versions.
- `make test` - basic phpt tests
- `make test-auto` - auto-instrumentation
- `make test-export` - otlp (`http/protobuf` + `grpc`) exporting to a local collector
- `make test-http` - spin up a cli-server and test auto-creation of root spans
- `make test-all` - all tests

For the `otlp` tests, be sure to `docker compose up -d collector` first.

### PHP 7.x

(tested back to 7.0)

Note that a couple of tests are skipped because they check for logs written during RSHUTDOWN/MSHUTDOWN, which doens't work in PHP 7.x.

### Debugging

There's a bunch of logging from the extension and the underlying opentelemetry-rust and dependencies. It's configurable via `.ini`:

`otel.log.level` -> `error` (default), `warn`, `info`, `debug` or `trace`

`otel.log.file` -> `/dev/stderr` (default), or another file/location of your choosing

If you really want to see what's going on, set the log level to `trace` and you'll get a lot of logs.

## SAPI support

### `cli`
http + grpc exporters work. Use .ini `otel.cli.create_root_span` to create a root span for this SAPI.

This should cover cli-based PHP runtimes (roadrunner, react, etc.), but has only been tested against RoadRunner.

### `cli-server`
http + grpc exporter works. Creates root span on RINIT.

### `apache2handler`
As above

### `cgi-fcgi`
As above

## Features

* Auto-instrumentation of userland (PHP8.0+) and internal (PHP8.2+) code, via zend_observer API (see `tests/auto/*`)
* Auto-instrumentation of userland code via `zend_execute_ex` (PHP 7.x)
* Start a span in RINIT, use `traceparent` headers, set HTTP response code in RSHUTDOWN
* TracerProvider created in RINIT (so that child processes have a working instance)
* Spans can be built through a SpanBuilder, some updates made (not all implemented yet), and `end()`ed
* Spans can be `activate()`d, and scope detached
* Spans export to stdout, otlp (grpc + http/protobuf)
* Batch and Simple span processors
* Get SpanContext from a Span
* Access "local root span"
* `memory` exporter for testing
* Support for shared hosting (ie one apache/fpm server with multiple sites), via `.env` files and `otel.dotenv.per_request` ini setting
* Disabling of auto-instrumentation via `.ini` setting `otel.auto.disabled_plugins`
  - eg `otel.auto.disabled_plugins=laminas,psr18`

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

## Multi-site support

### Vhosts

Multi-site using apache vhosts should "just work". You should set the required environment variables in the vhost config.
For example (untested):
```
<VirtualHost *:80>
    ServerName site1.example.com
    SetEnv OTEL_SERVICE_NAME my-service
    SetEnv OTEL_RESOURCE_ATTRIBUTES "service.namespace=site1,service.version=1.0"
    # Other config...
</VirtualHost>
```

If you cannot modify vhost config, you can also use the `.env` file support described below.

### No vhosts

If you have multiple sites on a single host (for example each application is a subdirectory of the web root), you can
use the `.env` file support to set the environment variables for each site. The extension will look for a `.env` file
for each request in the directory of the processed .php file (eg `/var/www/site1/public/index.php` -> `/var/www/site1/public/.env`). The .env files will be
checked for `OTEL_SERVICE_NAME` and `OTEL_RESOURCE_ATTRIBUTES` variables, and if they are set, they will be used to
configure the tracer.

## What doesn't work or isn't implemented? (todo list)

- Context storage - otel-rust doesn't support storing non-simple values, and context keys are created at compile time.
This will probably never work like opentelemetry-php.

## The future

Depending on what we can get to work, or not, this extension could go in a number of directions.

0. A simple, no-frills opentelemetry option for unsupported PHP versions (ie, 7.x)
1. An implementation of the opentelemetry-php API, backed by opentelemetry-rust API+SDK
2. Use opentelemetry-rust to support only auto-instrumentation a-la [SkyWalking](https://github.com/apache/skywalking-php/),
[Compass](https://github.com/skpr/compass/).
3. Implement an entirely new API, closer to opentelemetry-rust's (ie, don't try to match opentelemetry-php)
4. Some combination of the above (probably 1+2 or 2+3)
5. Abandon ship, chalk it up to experience
