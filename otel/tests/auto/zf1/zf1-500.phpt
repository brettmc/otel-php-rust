--TEST--
Test zf1 500
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

run_server('auto/zf1/public/index.php', $options, 'index/explode');
?>
--EXPECTF--
%A==== Response ====%A
==== Server Output ====%A
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "php:rinit"

	Name        : GET default/index/explode
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: 0000000000000000
	Kind        : Server
	Start time: %s
	End time: %s
	Status: Error { description: "something bad happened" }
	Attributes:
		 ->  url.full: String(Owned("/index/explode"))
		 ->  http.request.method: String(Owned("GET"))
		 ->  php.framework.name: String(Static("zf1"))
		 ->  php.framework.module.name: String(Owned("default"))
		 ->  php.framework.controller.name: String(Owned("index"))
		 ->  php.framework.action.name: String(Owned("explode"))
		 ->  http.response.status_code: I64(500)%A
	Events:
	Event #0
	Name      : exception
	Timestamp : %s
	Attributes:
		 ->  exception.message: String(Owned("something bad happened"))%A