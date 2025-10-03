--TEST--
Emit a log record with severity text
--EXTENSIONS--
otel
--ENV--
OTEL_TRACES_PROCESSOR=none
OTEL_LOGS_PROCESSOR=simple
OTEL_LOGS_EXPORTER=memory
--INI--
otel.log.level=warn
otel.cli.enabled=1
--FILE--
<?php
use OpenTelemetry\API\Globals;
use OpenTelemetry\API\Logs\LogRecord;
use OpenTelemetry\API\Logs\MemoryLogsExporter;

$severities = [
    "TRACE",
    "TRACE2",
    "TRACE3",
    "TRACE4",
    "DEBUG",
    "DEBUG2",
    "DEBUG3",
    "DEBUG4",
    "INFO",
    "INFO2",
    "INFO3",
    "INFO4",
    "WARN",
    "WARN2",
    "WARN3",
    "WARN4",
    "ERROR",
    "ERROR2",
    "ERROR3",
    "ERROR4",
    "FATAL",
    "FATAL2",
    "FATAL3",
    "FATAL4",
    "UNKNOWN", //should be ignored, warning emitted
];

$logger = Globals::loggerProvider()->getLogger("my_logger", '0.1', 'schema.url', ['one' => 1]);
foreach ($severities as $sev) {
    $record = new LogRecord('test-'.$sev);
    $record->setSeverityText($sev);
    $logger->emit($record);
}

var_dump(MemoryLogsExporter::count());
$exported = MemoryLogsExporter::getLogs();
foreach ($exported as $log) {
    var_dump($log['severity_text']);
}

?>
--EXPECTF--
%s[WARN]%sUnknown severity text: UNKNOWN
int(25)
string(5) "TRACE"
string(6) "TRACE2"
string(6) "TRACE3"
string(6) "TRACE4"
string(5) "DEBUG"
string(6) "DEBUG2"
string(6) "DEBUG3"
string(6) "DEBUG4"
string(4) "INFO"
string(5) "INFO2"
string(5) "INFO3"
string(5) "INFO4"
string(4) "WARN"
string(5) "WARN2"
string(5) "WARN3"
string(5) "WARN4"
string(5) "ERROR"
string(6) "ERROR2"
string(6) "ERROR3"
string(6) "ERROR4"
string(5) "FATAL"
string(6) "FATAL2"
string(6) "FATAL3"
string(6) "FATAL4"
string(0) ""
