--TEST--
Inject outgoing trace headers to psr-18 request
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=console
OTEL_SPAN_PROCESSOR=batch
--INI--
otel.log.level="warn"
otel.log.file="/dev/stdout"
--FILE--
<?php
use OpenTelemetry\API\Globals;
use Psr\Http\Client\ClientInterface;
use Psr\Http\Message\RequestInterface;
use Psr\Http\Message\ResponseInterface;
use Nyholm\Psr7\Request;
use Nyholm\Psr7\Response;

require 'vendor/autoload.php';

class MockHttpClient implements ClientInterface
{
	private ?RequestInterface $request = null;

    public function sendRequest(RequestInterface $request): ResponseInterface
    {
        $this->request = $request;

        // Return a mock response
        return new Response(200, [], "Mock response body");
    }

    public function getRequest(): ?RequestInterface
    {
        return $this->request;
    }
}

$request = new Request('OPTIONS', 'http://example.com:8000/resource/100', ['x-foo' => 'bar']);
$client = new MockHttpClient();

$span = Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('root')->startSpan();
$scope = $span->activate();

$response = $client->sendRequest($request);
$lastRequest = $client->getRequest();
assert($lastRequest->hasHeader('traceparent'));
assert($lastRequest->hasHeader('x-foo'));
$traceParent = $lastRequest->getHeader('traceparent')[0]; //todo assert matches active span
var_dump($traceParent);

$span->end();
$scope->detach();
?>
--EXPECTF--
string(55) "00-%s-%s-01"
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "php-auto-instrumentation"

	Name        : OPTIONS
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: %s
	Kind        : Client
	Start time: %s
	End time: %s
	Status: Unset
	Attributes:
		 ->  code.function.name: String(Owned("MockHttpClient::sendRequest"))
		 ->  code.file.path: String(Owned("/usr/src/myapp/tests/auto/psr18/psr18-inject.php"))
		 ->  code.line.number: I64(15)
		 ->  url.full: String(Owned("http://example.com:8000/resource/100"))
		 ->  url.scheme: String(Owned("http"))
		 ->  url.path: String(Owned("/resource/100"))
		 ->  server.address: String(Owned("example.com"))
		 ->  server.port: I64(8000)
		 ->  http.request.method: String(Owned("OPTIONS"))
		 ->  http.response.status_code: I64(200)
Span #1
	Instrumentation Scope
		Name         : "my_tracer"

	Name        : root
	TraceId     : %s
	SpanId      : %s
	TraceFlags  : TraceFlags(1)
	ParentSpanId: 0000000000000000
	Kind        : Internal
	Start time: %s
	End time: %s
	Status: Unset