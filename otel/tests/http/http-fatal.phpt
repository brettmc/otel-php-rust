--TEST--
Test HTTP span with fatal error
--EXTENSIONS--
otel
--FILE--
<?php
include dirname(__DIR__) . '/run-server.php';

$options = [
    "http" => [
        "method" => "GET",
    ]
];

run_server('http/server-fatal.php', $options);
?>
--EXPECTF--
%AHTTP/1.0 500 Internal Server Error
 in %A
==== Response ====
==== Server Output ====%A
[%s] PHP Fatal error:  Uncaught Error: Call to undefined function undefined_function() in %s
%A
Spans
Resource
%A
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
	Status       : Error { description: "" }
	Attributes:
		 ->  url.full: String(Owned("/"))
		 ->  http.request.method: String(Owned("GET"))
		 ->  http.response.status_code: I64(500)
	Events:
	Event #0
	Name      : exception
	Timestamp : %s
	Attributes:
		 ->  exception.type: String(Owned("PHP fatal error"))
		 ->  exception.message: String(Owned("Uncaught Error: Call to undefined function undefined_function() in %s"))
		 ->  exception.stacktrace: String(Owned("%stests/http/server-fatal.php:%d"))%A