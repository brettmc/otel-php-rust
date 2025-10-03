<?php
header('Content-Type: text/plain');
http_response_code(200);
var_dump($_SERVER['REQUEST_METHOD']);
var_dump($_SERVER['REQUEST_URI']);
