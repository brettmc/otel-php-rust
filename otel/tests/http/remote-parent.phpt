--TEST--
Test HTTP span with remote parent
--SKIPIF--
<?php
if (PHP_SAPI !== 'cli') {
    die('skip: Not running in CLI mode');
}
?>
--XFAIL--
is_remote not correct :(
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
Hello, World!
bool(true)
string(32) "e77388f01a826e2de7afdcd1eefc034e"
string(16) "%s"
==== Server Output ====
[%s] PHP %s Development Server (%s) started
[%s] %s Accepted
Spans
Resource
%A
Span #0
	Instrumentation Scope
%A
	Name        : HTTP GET
	TraceId     : e77388f01a826e2de7afdcd1eefc034e
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: d6ba64af4fa59b65
	Kind        : Server
	Start time: %s
	End time: %s
	Status: Unset
	Attributes:
		 ->  http.response.status_code: I64(200)
[%s] %s Closing