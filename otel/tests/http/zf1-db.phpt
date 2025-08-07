--TEST--
Test zf1
--SKIPIF--
<?php
if (PHP_SAPI !== 'cli') {
    die('skip: Not running when not in CLI mode');
}
?>
--EXTENSIONS--
otel
--FILE--
<?php
include 'run-server.php';

$options = [
    "http" => [
        "method" => "GET",
    ]
];

run_server('../auto/zf1/public/index.php', $options, "index/query");
?>
--EXPECTF--
==== Response ====
%A
string(16) "controller=index"
string(12) "action=query"
int(2)
==== Server Output ====%A
Spans
Resource
%A
Span #0
	Instrumentation Scope
%A
	Name        : GET default/index/query
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: %s
	Kind        : Server
	Start time: %s
	End time: %s
	Status: Unset
	Attributes:
		 ->  url.full: String(Owned("/index/query"))
		 ->  http.request.method: String(Owned("GET"))%A
%A
Span #1
Instrumentation Scope
%A
	Name        : Statement::execute
	%A
	Kind        : Client
	%A
	Attributes:
         ->  db.system: String(Owned("sqlite"))
         ->  db.name: String(Owned("test.sqlite"))
         ->  db.statement: String(Owned("select * from users"))
         ->  db.operation: String(Owned("query"))
         ->  db.connection_string: String(Owned("dbname=/data/test.sqlite"))%A