--TEST--
Test zf1 200 ok
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

run_server('auto/zf1/public/index.php', $options);
?>
--EXPECTF--
==== Response ====
%AHello from Zend Framework 1%A
==== Server Output ====%A
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "php:rinit"

	Name        : GET default/index/index
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
		 ->  php.framework.name: String(Static("zf1"))
		 ->  php.framework.module.name: String(Owned("default"))
		 ->  php.framework.controller.name: String(Owned("index"))
		 ->  php.framework.action.name: String(Owned("index"))
		 ->  http.response.status_code: I64(200)%A