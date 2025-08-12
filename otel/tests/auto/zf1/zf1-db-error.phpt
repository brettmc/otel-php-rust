--TEST--
Test zf1 Zend_Db error
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

run_server('auto/zf1/public/index.php', $options, 'users/broken');
?>
--EXPECTF--
Warning: %s HTTP/1.1 500
 in %s
==== Response ====
==== Server Output ====%A
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "php.otel.zf1"

	Name        : Statement::prepare
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: %s
	Kind        : Client
	Start time: %s
	End time: %s
	Status: Error { description: "The ibm driver is not currently installed" }
	Attributes:
		 ->  code.function.name: String(Owned("Zend_Db_Adapter_Pdo_Ibm::prepare"))
		 ->  code.file.path: String(Owned("%s/zend-db/library/Zend/Db/Adapter/Pdo/Ibm.php"))
		 ->  code.line.number: I64(%d)
		 ->  db.query.text: String(Owned("select * from users"))
	Events:
	Event #0
	Name      : exception
	Timestamp : %s
	Attributes:
		 ->  exception.message: String(Owned("The ibm driver is not currently installed"))
		 ->  exception.type: String(Owned("Zend_Db_Adapter_Exception"))
		 ->  exception.stacktrace: String(Owned("%s"))
Spans
Span #0
	Instrumentation Scope
		Name         : "php:rinit"

	Name        : GET default/users/broken
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: 0000000000000000
	Kind        : Server
	Start time: %s
	End time: %s
	Status: Error { description: "The ibm driver is not currently installed" }
	Attributes:
		 ->  url.full: String(Owned("/users/broken"))
		 ->  http.request.method: String(Owned("GET"))
		 ->  php.framework.name: String(Static("zf1"))
		 ->  php.framework.module.name: String(Owned("default"))
		 ->  php.framework.controller.name: String(Owned("users"))
		 ->  php.framework.action.name: String(Owned("broken"))
		 ->  http.response.status_code: I64(500)
	Events:
	Event #0
	Name      : exception
	Timestamp : %s
	Attributes:
		 ->  exception.message: String(Owned("The ibm driver is not currently installed"))%A
