--TEST--
Test HTTP span with POST
--SKIPIF--
<?php
if (PHP_SAPI !== 'cli') {
    die('skip: Not running in CLI mode');
}
?>
--EXTENSIONS--
otel
--FILE--
<?php
include 'run-server.php';

$options = [
    "http" => [
        "method" => "POST",
    ]
];

run_server('server-post.php', $options);
?>
--EXPECTF--
==== Response ====
string(4) "POST"
==== Server Output ====
%A
Spans
Resource
%A
Span #0
	Instrumentation Scope
%A
	Name        : HTTP POST
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: 0000000000000000
	Kind        : Server
	Start time: %s
	End time: %s
	Status: Unset
	Attributes:
		 ->  http.response.status_code: I64(201)
%A