--TEST--
Let auto root span create a span, then modify it via Span::getLocalRoot
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
        "method" => "PUT",
    ]
];

run_server('http/server-modify-local-root-span.php', $options);
?>
--EXPECTF--
==== Response ====
==== Server Output ====%A
Spans
Resource
%A
Span #0
	Instrumentation Scope
%A
	Name        : I was updated via LocalRootSpan
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
		 ->  http.request.method: String(Owned("PUT"))
		 ->  http.response.status_code: I64(201)%A