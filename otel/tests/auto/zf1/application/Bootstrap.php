<?php
class Bootstrap extends Zend_Application_Bootstrap_Bootstrap
{
    public function _initRoutes()
    {

        $frontController = Zend_Controller_Front::getInstance();
        $router = $frontController->getRouter();

        $route = new Zend_Controller_Router_Route (
            'bar/:id',
            [
                'controller' => 'index',
                'action' => 'foo',
            ]
        );

        $router->addRoute('bar', $route);
    }
}