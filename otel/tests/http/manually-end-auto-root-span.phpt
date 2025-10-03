--TEST--
Let auto root span create a span, then manually end it
--DESCRIPTION--
Since root span has ended, RSHUTDOWN will not be able to modify it
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
        "method" => "OPTIONS",
    ]
];

run_server('http/server-end-root-span.php', $options);
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
	Name         : OPTIONS
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
		 ->  http.request.method: String(Owned("OPTIONS"))%A
