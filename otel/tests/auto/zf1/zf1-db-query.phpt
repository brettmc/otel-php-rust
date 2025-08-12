--TEST--
Test zf1 200 ok
--EXTENSIONS--
otel
--FILE--
<?php
// TODO this is a placeholder for Zend_Db tests, which are not yet implemented
include dirname(__DIR__, 2) . '/run-server.php';

$options = [
    "http" => [
        "method" => "GET",
    ]
];

run_server('auto/zf1/public/index.php', $options, 'users/list');
?>
--EXPECTF--
==== Response ====
%A<title>Users</title>%A
==== Server Output ====%A
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "php:rinit"

	Name        : GET default/users/list
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: 0000000000000000
	Kind        : Server
	Start time: %s
	End time: %s
	Status: Unset
	Attributes:
		 ->  url.full: String(Owned("/users/list"))
		 ->  http.request.method: String(Owned("GET"))
		 ->  php.framework.name: String(Static("zf1"))
		 ->  php.framework.module.name: String(Owned("default"))
		 ->  php.framework.controller.name: String(Owned("users"))
		 ->  php.framework.action.name: String(Owned("list"))
		 ->  http.response.status_code: I64(200)%A
