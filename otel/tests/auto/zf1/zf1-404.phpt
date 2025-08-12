--TEST--
Test zf1 404
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

run_server('auto/zf1/public/index.php', $options, 'does-not-exist/index');
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

	Name        : GET default/does-not-exist/index
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: 0000000000000000
	Kind        : Server
	Start time: %s
	End time: %s
	Status: Unset
	Attributes:
		 ->  url.full: String(Owned("/does-not-exist/index"))
		 ->  http.request.method: String(Owned("GET"))
		 ->  php.framework.name: String(Static("zf1"))
		 ->  php.framework.module.name: String(Owned("default"))
		 ->  php.framework.controller.name: String(Owned("does-not-exist"))
		 ->  php.framework.action.name: String(Owned("index"))
		 ->  http.response.status_code: I64(404)%A
	Events:
	Event #0
	Name      : exception
	Timestamp : %s
	Attributes:
		 ->  exception.message: String(Owned("Invalid controller specified (does-not-exist)"))%A