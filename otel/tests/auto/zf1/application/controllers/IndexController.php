<?php
class IndexController extends Zend_Controller_Action
{
    public function indexAction()
    {
        // This will render views/scripts/index/index.phtml
        echo "IndexController::indexAction";
    }

    public function fooAction()
    {
        echo "IndexController::fooAction";
        $request = $this->getRequest();
        $controller = $request->getControllerName();
        $module = $request->getModuleName();
        echo 'Controller: ' . htmlspecialchars($controller) . '<br>';
        echo 'Module: ' . htmlspecialchars($module) . '<br>';
        var_dump(get_class($request));
    }
}