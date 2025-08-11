--TEST--
Test HTTP span with 500 error response
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

run_server('http/server-error.php', $options);
?>
--EXPECTF--
Warning: %S HTTP request failed! HTTP/%s 500 Internal Server Error
 in %s/run-server.php on line %d
==== Response ====%A
==== Server Output ====%A
Spans
Resource
%A
Span #0
	Instrumentation Scope
%A
	Name        : GET
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: 0000000000000000
	Kind        : Server
	Start time: %s
	End time: %s
	Status: Error { description: "" }
	Attributes:
		 ->  url.full: String(Owned("/"))
		 ->  http.request.method: String(Owned("GET"))
		 ->  http.response.status_code: I64(500)%A