<?php
header('Content-Type: text/plain');
http_response_code(201);
var_dump($_SERVER['REQUEST_METHOD']);
