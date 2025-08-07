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

    public function queryAction()
    {
         var_dump('controller=index');
         var_dump('action=query');
         $dbname = APPLICATION_PATH . '/../data/test.sqlite';
         $db = new Zend_Db_Adapter_Pdo_Sqlite(array('dbname' => $dbname));
         $stmt = $db->prepare('select * from users');
         $stmt->execute();
         $result = $stmt->fetchAll();
         var_dump(count($result));
    }
}