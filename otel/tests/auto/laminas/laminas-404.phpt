--TEST--
Test laminas 404 not found
--EXTENSIONS--
otel
--SKIPIF--
<?php
if (PHP_VERSION_ID < 70100 || PHP_VERSION_ID >= 80300) {
    die('skip requires PHP 7.1 -> 8.2');
}
?>
--FILE--
<?php
include dirname(__DIR__, 2) . '/run-server.php';

$options = [
    "http" => [
        "method" => "GET",
    ]
];

run_server('auto/laminas/public/index.php', $options, 'does-not-exist');
?>
--EXPECTF--
Warning: %s HTTP request failed! HTTP/%s 404 Not Found
 in %s
==== Response ====
==== Server Output ====%A
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "php.otel.auto.laminas"

	Name         : Application::run
	TraceId      : %s
	SpanId       : %s
	TraceFlags   : TraceFlags(1)
	ParentSpanId : %s
	Kind         : Internal
	Start time   : %s
	End time     : %s
	Status       : Error { description: "error-router-no-match" }
	Attributes:
		 ->  code.function.name: String(Owned("Laminas\\Mvc\\Application::run"))
		 ->  code.file.path: String(Owned("%s/laminas/laminas-mvc/src/Application.php"))
		 ->  code.line.number: I64(%d)
	Events:
	Event #0
	Name      : exception
	Timestamp : %s
	Attributes:
		 ->  exception.message: String(Owned("error-router-no-match"))
Spans
Span #0
	Instrumentation Scope
		Name         : "php:rinit"

	Name         : GET
	TraceId      : %s
	SpanId       : %s
	TraceFlags   : TraceFlags(1)
	ParentSpanId : None (root span)
	Kind         : Server
	Start time   : %s
	End time     : %s
	Status       : Unset
	Attributes:
		 ->  url.full: String(Owned("/does-not-exist"))
		 ->  http.request.method: String(Owned("GET"))
		 ->  php.framework.name: String(Static("laminas"))
		 ->  http.response.status_code: I64(404)%A