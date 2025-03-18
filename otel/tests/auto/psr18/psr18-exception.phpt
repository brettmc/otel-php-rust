--TEST--
Handle exception from psr18 sendRequest
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
use Psr\Http\Client\ClientExceptionInterface;
use Psr\Http\Message\RequestInterface;
use Psr\Http\Message\ResponseInterface;
use Nyholm\Psr7\Request;
use Nyholm\Psr7\Response;

require 'vendor/autoload.php';

class MyClientException extends \Exception implements ClientExceptionInterface {}

class MockHttpClient implements ClientInterface
{
	private ?RequestInterface $request = null;

    public function sendRequest(RequestInterface $request): ResponseInterface
    {
        throw new MyClientException('something went wrong', 500);
    }
}

$request = new Request('GET', 'http://example.com');
$client = new MockHttpClient();

$span = Globals::tracerProvider()->getTracer('my_tracer')->spanBuilder('root')->startSpan();
$scope = $span->activate();

try {
	$response = $client->sendRequest($request);
} catch (ClientExceptionInterface $ce) {
	var_dump($ce->getMessage());
}
$span->end();
$scope->detach();
?>
--EXPECTF--
string(20) "something went wrong"
Spans
Resource
%A
Span #0
	Instrumentation Scope
		Name         : "php-auto-instrumentation"

	Name        : GET
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
		 ->  code.file.path: String(Owned("/usr/src/myapp/tests/auto/psr18/psr18-exception.php"))
		 ->  code.line.number: I64(18)
		 ->  url.full: String(Owned("http://example.com"))
		 ->  url.scheme: String(Owned("http"))
		 ->  url.path: String(Owned(""))
		 ->  server.address: String(Owned("example.com"))
		 ->  http.request.method: String(Owned("GET"))
	Events:
	Event #0
	Name      : exception
	Timestamp : %s
	Attributes:
		 ->  exception.message: String(Owned("something went wrong"))
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