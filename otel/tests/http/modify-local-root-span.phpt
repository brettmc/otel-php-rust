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
include 'run-server.php';

$options = [
    "http" => [
        "method" => "PUT",
    ]
];

run_server('server-modify-local-root-span.php', $options);
?>
--EXPECTF--
==== Response ====
==== Server Output ====
[%s] PHP %s Development Server (%s) started
[%s] %s Accepted
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
		 ->  php.sapi.name: String(Owned("cli-server"))
		 ->  url.full: String(Owned("/"))
		 ->  http.request.method: String(Owned("PUT"))
		 ->  http.response.status_code: I64(201)
[%s] %s Closing