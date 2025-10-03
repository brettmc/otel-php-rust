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
include dirname(__DIR__) . '/run-server.php';

$options = [
    "http" => [
        "method" => "POST",
    ]
];

run_server('http/server-post.php', $options);
?>
--EXPECTF--
==== Response ====
string(4) "POST"
==== Server Output ====%A
Spans
Resource
%A
Span #0
	Instrumentation Scope
%A
	Name         : POST
	TraceId      : %s
	SpanId       : %s
	TraceFlags   : TraceFlags(1)
	ParentSpanId : None (root span)
	Kind         : Server
	Start time   : %s
	End time     : %s
	Status       : Unset
	Attributes:
		 ->  url.full: String(Owned("/"))
		 ->  http.request.method: String(Owned("POST"))
		 ->  http.response.status_code: I64(201)%A