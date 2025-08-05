<?php

define('APPLICATION_PATH', realpath(__DIR__ . '/../application'));
require_once __DIR__ . '/../vendor/autoload.php';

// Set include path for ZF1
set_include_path(implode(PATH_SEPARATOR, [
    __DIR__ . '/../vendor/zf1s/zf1/library',
    get_include_path(),
]));

$application = new Zend_Application(
    'development',
    __DIR__ . '/../application/configs/application.ini'
);
$application->bootstrap()->run();
