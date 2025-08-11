--TEST--
Test laminas 200 ok
--EXTENSIONS--
otel
--FILE--
<?php
include dirname(__DIR__, 2) . '/run-server.php';

$options = [
    "http" => [
        "method" => "GET",
    ]
];

run_server('auto/laminas/public/index.php', $options);
?>
--EXPECTF--
==== Response ====
%A
        <title>Laminas MVC Skeleton</title>
%A
==== Server Output ====
%A
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
	Status: Unset
	Attributes:
		 ->  code.function.name: String(Owned("Laminas\\Mvc\\Application::run"))
		 ->  code.file.path: String(Owned("%s/laminas/laminas-mvc/src/Application.php"))
		 ->  code.line.number: I64(%d)
Spans
Span #0
	Instrumentation Scope
		Name         : "php:rinit"

	Name        : GET home
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: 0000000000000000
	Kind        : Server
	Start time: %s
	End time: %s
	Status: Unset
	Attributes:
		 ->  url.full: String(Owned("/"))
		 ->  http.request.method: String(Owned("GET"))
		 ->  php.framework.name: String(Static("laminas"))
		 ->  php.framework.controller.name: String(Owned("Application\\Controller\\IndexController"))
		 ->  php.framework.action.name: String(Owned("index"))
		 ->  http.response.status_code: I64(200)
%A