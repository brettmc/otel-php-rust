<?php
class ErrorController extends Zend_Controller_Action
{
    public function errorAction()
    {
        $errors = $this->_getParam('error_handler');
        if ($errors->type == Zend_Controller_Plugin_ErrorHandler::EXCEPTION_NO_ROUTE ||
            $errors->type == Zend_Controller_Plugin_ErrorHandler::EXCEPTION_NO_CONTROLLER ||
            $errors->type == Zend_Controller_Plugin_ErrorHandler::EXCEPTION_NO_ACTION) {
            $this->getResponse()->setHttpResponseCode(404);
        } else {
            $this->getResponse()->setHttpResponseCode(500);
        }
    }
}