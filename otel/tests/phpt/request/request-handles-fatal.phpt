--TEST--
Auto root span handles fatal errors
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
OTEL_SPAN_PROCESSOR=simple
--INI--
otel.log.level="error"
otel.log.file="/dev/stdout"
otel.cli.create_root_span="On"
otel.cli.enabled=1
--FILE--
<?php
undefined_function();
?>
--EXPECTF--
Fatal error: Uncaught Error: Call to undefined function undefined_function() in %s
Stack trace:
#0 {main}
  thrown in %s
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "php:rinit"

	Name         : php:cli
	TraceId      : %s
	SpanId       : %s
	TraceFlags   : TraceFlags(1)
	ParentSpanId : None (root span)
	Kind         : Server
	Start time   : %s
	End time     : %s
	Status       : Unset
	Attributes:
		 ->  url.full: String(Owned(""))
		 ->  http.request.method: String(Owned(""))