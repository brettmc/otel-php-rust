<?php
class IndexController extends Zend_Controller_Action
{
    public function indexAction()
    {
        // This will render views/scripts/index/index.phtml
        echo "IndexController::indexAction";
    }

    public function explodeAction()
    {
        throw new \RuntimeException('something bad happened');
    }
}