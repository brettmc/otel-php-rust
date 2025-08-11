--TEST--
Test laminas 500 error
--EXTENSIONS--
otel
--SKIPIF--
<?php
if (PHP_VERSION_ID < 70100) {
    die('skip requires PHP 7.1 or higher');
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

run_server('auto/laminas/public/index.php', $options, 'tick/tick');
?>
--EXPECTF--
Warning: file_get_contents(%stick/tick): failed to open stream: HTTP request failed! HTTP/%s 500 Internal Server Error
 in %s
==== Response ====
==== Server Output ====
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "php.otel.auto.laminas"

	Name        : Application::run
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: %s
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Error { description: "" }
	Attributes:
		 ->  code.function.name: String(Owned("Laminas\\Mvc\\Application::run"))
		 ->  code.file.path: String(Owned("%s/laminas/laminas-mvc/src/Application.php"))
		 ->  code.line.number: I64(%d)
	Events:
	Event #0
	Name      : exception
	Timestamp : %s
	Attributes:
		 ->  exception.message: String(Owned("boom"))
		 ->  exception.type: String(Owned("RuntimeException"))
		 ->  exception.stacktrace: String(Owned("%s"))
Spans
Span #0
	Instrumentation Scope
		Name         : "php:rinit"

	Name        : GET ticktick
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: 0000000000000000
	Kind        : Server
	Start time: %s
	End time: %s
	Status: Error { description: "" }
	Attributes:
		 ->  url.full: String(Owned("/tick/tick"))
		 ->  http.request.method: String(Owned("GET"))
		 ->  php.framework.name: String(Static("laminas"))
		 ->  php.framework.controller.name: String(Owned("Application\\Controller\\ThrowsErrorController"))
		 ->  php.framework.action.name: String(Owned("boom"))
		 ->  http.response.status_code: I64(500)