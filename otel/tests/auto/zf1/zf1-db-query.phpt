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
<h1>layout</h1>
<title>Users</title>
<pre>array(2) {
  [0]=>
  array(3) {
    ["id"]=>
    string(1) "1"
    ["name"]=>
    string(3) "sam"
    ["email"]=>
    string(15) "sam@example.com"
  }
  [1]=>
  array(3) {
    ["id"]=>
    string(1) "2"
    ["name"]=>
    string(4) "emma"
    ["email"]=>
    string(16) "emma@example.com"
  }
}
</pre>string(21) "Zend_Db_Statement_Pdo"
%A
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
	Status: Unset
	Attributes:
		 ->  code.function.name: String(Owned("Zend_Db_Adapter_Pdo_Abstract::prepare"))
		 ->  code.file.path: String(Owned("%s/zend-db/library/Zend/Db/Adapter/Pdo/Abstract.php"))
		 ->  code.line.number: I64(%d)
		 ->  db.query.text: String(Owned("select * from users"))
Spans
Span #0
	Instrumentation Scope
		Name         : "php.otel.zf1"

	Name        : Statement::execute
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: %s
	Kind        : Client
	Start time: %s
	End time: %s
	Status: Unset
	Attributes:
		 ->  code.function.name: String(Owned("Zend_Db_Statement::execute"))
		 ->  code.file.path: String(Owned("%s/zend-db/library/Zend/Db/Statement.php"))
		 ->  code.line.number: I64(%d)
Spans
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