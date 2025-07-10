--TEST--
Inject outgoing trace headers to psr-18 request
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_EXPORTER=memory
OTEL_SPAN_PROCESSOR=simple
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
use OpenTelemetry\API\Trace\SpanExporter\Memory;

require 'vendor/autoload.php';

class MockHttpClient implements ClientInterface
{
	private $request = null;

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

$span = Globals::tracerProvider()->getTracer('my_tracer', '0.1', 'schema.url')->spanBuilder('root')->startSpan();
$scope = $span->activate();

$response = $client->sendRequest($request);
$lastRequest = $client->getRequest();
assert($lastRequest->hasHeader('traceparent'));
assert($lastRequest->hasHeader('x-foo'));
$traceParent = $lastRequest->getHeader('traceparent')[0]; //todo assert matches active span
var_dump($traceParent);

$span->end();
$scope->detach();

assert(Memory::count() === 2);
$spans = Memory::getSpans();
$psr18 = $spans[0];
var_dump($psr18['name']);
var_dump($psr18['span_kind']);
var_dump($psr18['attributes']);
?>
--EXPECTF--
string(55) "00-%s-%s-01"
string(7) "OPTIONS"
string(6) "Client"
array(10) {
  ["code.function.name"]=>
  string(27) "MockHttpClient::sendRequest"
  ["code.file.path"]=>
  string(48) "/usr/src/myapp/tests/auto/psr18/psr18-inject.php"
  ["code.line.number"]=>
  int(%d)
  ["url.full"]=>
  string(36) "http://example.com:8000/resource/100"
  ["url.scheme"]=>
  string(4) "http"
  ["url.path"]=>
  string(13) "/resource/100"
  ["server.address"]=>
  string(11) "example.com"
  ["server.port"]=>
  int(8000)
  ["http.request.method"]=>
  string(7) "OPTIONS"
  ["http.response.status_code"]=>
  int(200)
}
