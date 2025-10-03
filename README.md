# OpenTelemetry + Rust

## Intro

This is a prototype PHP extension, using [phper](https://github.com/phper-framework/phper)
to expose [opentelemetry-rust](https://opentelemetry.io/docs/languages/rust/) via PHP
classes and implement auto-instrumentation.

The initial idea was to implement the PHP API in an extension, which could be a
drop-in replacement for the core of opentelemetry-php. Since 3rd parties are
strongly encouraged to only depend on the API, this should be all they need.

As it's slowly evolved and things have changed in the official opentelemetry-php
implementation, the focus has shifted to being a small and fast opentelemetry implementation
for legacy PHP versions that currently do not have an official way to use
OpenTelemetry.

I've focussed development whilst thinking about a large legacy shared PHP host
at my day job, where we have hundreds of small to medium PHP applications running
on a single server (99% running as sub-directories in one Apache vhost), and we
want to be able to trace them without modifying the code. There is a mix of frameworks, and some
applications that pre-date modern frameworks.

## Supported PHP versions

This works on all PHP versions supported by phper (7.0+). I've mostly tested against
`8.4`, `7.4` and some light testing against `7.0`. Auto-instrumentation is implemented
via the `zend_observer` API for PHP 8.0+, and via `zend_execute_ex` + `zend_execute_internal`
for PHP 7.x.

### PHP 7.x

Note that writing to stdout/stderr during MSHUTDOWN doesn't seem to work in PHP 7.x, so
console exporting and log writing do not work for this stage. This mostly affects tests,
and OTLP exporting works as expected.

## Installation

All build approaches require `llvm-dev`, `libclang-dev` (at least 9.0), rust compiler and cargo.

### PIE (PHP Installer for Extensions)

_Only works for PHP versions supported by PIE (8.1+)_

`php pie.phar install brettmc/otel-php-rust:<version>`

### Manual

```shell
git clone <this repository>
cd otel-php-rust
git checkout <version>
make build
make install
```

## Debugging

There's a bunch of logging from the extension and the underlying opentelemetry-rust and dependencies. It's configurable via `.ini`:

`otel.log.level` -> `error` (default), `warn`, `info`, `debug` or `trace`

`otel.log.file` -> `/dev/stderr` (default), or another file/location of your choosing

If you really want to see what's going on, set the log level to `trace` and you'll get a lot of logs.

In PHP 7.x, logging to stdout/stderr during MSHUTDOWN doesn't work, so you will need to set `otel.log.file`
to a file location if you want to see logs from shutdown phase.

## SAPI support

Tested against `cli-server`, `apache2handler`, `cgi-fcgi` and `cli`.

### `cli`
Does not auto-create a root span by default, use .ini `otel.cli.create_root_span` to enable.

This should cover cli-based PHP runtimes (roadrunner, react, etc.), but has only been tested against RoadRunner.

## Features

* Auto-instrumentation of userland and internal code
  - using either Zend Observer API (PHP 8.0+), or zend_execute_ex/zend_execute_internal (PHP 7.x)
* Start a span in RINIT, use `traceparent` headers, set HTTP response code in RSHUTDOWN
* Exclude URLs from being traced: `OTEL_PHP_EXCLUDED_URLS=/health*,/ping`
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
* Configure OTEL_SERVICE_NAME, OTEL_RESOURCE_ATTRIBUTES and OTEL_DISABLED via .env (for multiple applications on the same host, you can override the general server environment variables)
* Some initial auto-instrumentation plugins:
  - Laminas
  - Zend Framework 1
  - PSR-18 HTTP client

## Configuration

### .ini
| Name                       | Default        | Description |
|----------------------------|----------------| ----------- |
| otel.log.level             | error          | Log level: error, warn, debug, trace |
| otel.log.file              | /dev/stderr    | Log destination: file or stdout/stderr |
| otel.cli.create_root_span  | false          | Whether to create a root span for CLI requests |
| otel.cli.enabled           | false          | Whether to enable OpenTelemetry for CLI requests |
| otel.env.set_from_server | false | Whether to set OTEL_* environment variables into the environment |
| otel.env.dotenv.enabled    | false          | Whether to load .env files per request |
| otel.auto.enabled          | true | Auto-instrumentation enabled |
| otel.auto.disabled_plugins | _empty string_ | A list of auto-instrumentation plugins to disable, comma-separated |

If either `otel.env.set_from_server` or `otel.env.dotenv.enabled` is set to true, the extension will back up the current
environment variables on RINIT, and restore them on RSHUTDOWN.

### Environment variables

All official OpenTelemetry [SDK configuration](https://github.com/open-telemetry/opentelemetry-specification/blob/v1.46.0/specification/configuration/sdk-environment-variables.md#general-sdk-configuration)
environment variables understood by [opentelemetry-rust](https://github.com/open-telemetry/opentelemetry-rust).

If variables are not set in the process environment (eg via apache [SetEnv](https://httpd.apache.org/docs/current/env.html)),
then set `otel.env.set_from_server` via php.ini, and `OTEL_*` variables from `$_SERVER` will be set in the environment.

### .env files

`OTEL_SERVICE_NAME`, `OTEL_RESOURCE_ATTRIBUTES` and `OTEL_SDK_DISABLED` can be set in a `.env` file. Other variables should be
set in the environment (todo: could be relaxed to allow setting all OpenTelemetry SDK configuration variables in the `.env` file).

## Usage

### Auto-instrumentation

By installing the extension and providing the basic [SDK configuration](https://github.com/open-telemetry/opentelemetry-specification/blob/v1.46.0/specification/configuration/sdk-environment-variables.md#general-sdk-configuration)
that opentelemetry expects, each HTTP request will generate an HTTP server root span. There are some initial
auto-instrumentation plugins for some legacy frameworks.

### Manual instrumentation

#### Tracing

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

## Logs
```php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Logs\LogRecord;

$logger = Globals::loggerProvider()->getLogger("my_logger", '0.1');
$record = new LogRecord('hello otel');
$record
    ->setSeverityNumber(9) //info
    ->setSeverityText('Info')
    ->setEventName('my_event')
    ->setTimestamp((int) (microtime(true) * 1e9))
    ->setAttributes([
        'a_bool' => true,
        'an_int' => 1,
        'a_float' => 1.1,
        'a_string' => 'foo',
        'string_array' => ['one', 'two', 'three'],
        'int_array' => [1, 2, 3],
        'float_array' => [1.1, 2.2, 3.3],
        'bool_array' => [true, false, true],
    ]);
$logger->emit($record);
```

Note that if there is an active span when the log record is emitted, the span context
will be associated with the log record.

## Plugins

Mostly the framework plugins hook in to routing mechanism of that framework, update
the root span's name to something more meaningful, and add some attributes.

A couple of non-standard attributes are added if the data is available:
`php.framework.name`, `php.framework.module.name`, `php.framework.controller.name`,
`php.framework.action.name`.

### Laminas

Hooks `Laminas\Mvc\MvcEvent::setRouteMatch`. Sets framework name, and uses the `RouteMatch` to set
module, controller and action names.
Hooks some of the `Laminas\Db` methods to create CLIENT spans for database queries.

### Zend Framework 1

Hooks `Zend_Controller_Router_Interface::route`. Sets framework name, and uses the
`Zend_Controller_Request_Abstract` to set module, controller and action names.

Hooks some Zend_Db methods to create CLIENT spans for database queries.

### Psr-18

Hooks `Psr\Http\Client\ClientInterface::sendRequest`, creates a CLIENT span and
injects the `traceparent` header into outgoing HTTP requests.

## Multi-site support

### Vhosts

If providing configuration via Apache `SetEnv` directives, or FPM `env[OTEL_*]` variables, you should
enable the `otel.env.set_from_server` setting in your php.ini, so that the extension
will set the environment variables from the server environment on RINIT, and restore them on RSHUTDOWN.

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

### No vhosts or multiple applications per vhost

If you have multiple sites on a single host (for example each application is a subdirectory of the web root), you can
use the `.env` file support to set the environment variables for each site.

During request startup (RINIT), the extension will look for a `.env` file in the directory of the
processed .php file (eg `/var/www/site1/public/index.php` -> `/var/www/site1/public/.env`),
and traverse up until `DOCUMENT_ROOT` is reached. If a .env file is found, it will be checked for
`OTEL_SDK_DISABLED`, `OTEL_SERVICE_NAME` and `OTEL_RESOURCE_ATTRIBUTES` variables, and if they are set,
they will be set in the current environment, and the original values restored at RSHUTDOWN.

NB that the  modified environment variables may not be reflected in `$_SERVER`, but should be visible via
`getenv()`.

### Opt-in or opt-out

OpenTelemetry can be either opt-in, or opt-out, using environment variables and `.env` files.

If you want to disable OpenTelemetry by default, and enable it for specific applications, you can set
`OTEL_DISABLED=true` in the server environment, and then set `OTEL_DISABLED=false` in the `.env` file
for each application you want to enable observability for.

If you want to enable OpenTelemetry by default, and disable it for specific applications, you can set
`OTEL_DISABLED=false` in the server environment, and then set `OTEL_DISABLED=true` in the `.env` file
for each application you want to disable observability for.

## What doesn't work or isn't implemented?

- Context storage - otel-rust doesn't support storing non-simple values, and context keys are created at compile time.
 This will probably never work like opentelemetry-php.
- It could use more interfaces to align with the official OpenTelemetry PHP API