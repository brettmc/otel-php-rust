--TEST--
dotenv can set request root span instrumentation scope
--EXTENSIONS--
otel
--INI--
otel.cli.enable=On
otel.cli.create_root_span=On
otel.dotenv.per_request=On
otel.log.level=debug
--ENV--
OTEL_TRACES_EXPORTER=console
OTEL_SPAN_PROCESSOR=simple
--FILE--
--EXPECTF--
%A
Span #0
	Instrumentation Scope
		Name         : "my-framework"
		Version  : "1.0.0"
%A