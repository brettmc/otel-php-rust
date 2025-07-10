--TEST--
Handle exception from psr18 sendRequest
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
use Psr\Http\Client\ClientExceptionInterface;
use Psr\Http\Message\RequestInterface;
use Psr\Http\Message\ResponseInterface;
use Nyholm\Psr7\Request;
use Nyholm\Psr7\Response;
use OpenTelemetry\API\Trace\SpanExporter\Memory;

require 'vendor/autoload.php';

class MyClientException extends \Exception implements ClientExceptionInterface {}

class MockHttpClient implements ClientInterface
{
	private $request = null;

    public function sendRequest(RequestInterface $request): ResponseInterface
    {
        throw new MyClientException('something went wrong', 500);
    }
}

$request = new Request('GET', 'http://example.com');
$client = new MockHttpClient();

try {
	$response = $client->sendRequest($request);
} catch (ClientExceptionInterface $ce) {
	var_dump($ce->getMessage());
}
$span = Memory::getSpans()[0];
var_dump($span['events']);
?>
--EXPECTF--
string(20) "something went wrong"
array(1) {
  [0]=>
  array(3) {
    ["name"]=>
    string(9) "exception"
    ["timestamp"]=>
    int(%d)
    ["attributes"]=>
    array(1) {
      ["exception.message"]=>
      string(20) "something went wrong"
    }
  }
}