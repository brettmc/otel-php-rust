--TEST--
Test HTTP span with remote parent
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
        "method" => "GET",
        "header" => "traceparent: 00-e77388f01a826e2de7afdcd1eefc034e-d6ba64af4fa59b65-01\r\n"
    ]
];

run_server('server-remote-parent.php', $options);
?>
--EXPECTF--
==== Response ====
string(32) "e77388f01a826e2de7afdcd1eefc034e"
string(16) "%s"
bool(false)
==== Server Output ====%A
Spans
Resource
%A
Span #0
	Instrumentation Scope
%A
	Name        : GET
	TraceId     : e77388f01a826e2de7afdcd1eefc034e
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: d6ba64af4fa59b65
	Kind        : Server
	Start time: %s
	End time: %s
	Status: Unset
	Attributes:
		 ->  php.sapi.name: String(Owned("cli-server"))
		 ->  url.full: String(Owned("/"))
		 ->  http.request.method: String(Owned("GET"))%A