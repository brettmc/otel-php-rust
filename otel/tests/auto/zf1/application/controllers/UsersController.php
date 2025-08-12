<?php
use Zend_Db_Adapter_Pdo_Ibm as Ibm;

class UsersController extends Zend_Controller_Action
{
    public function listAction()
    {
        // This will render views/scripts/index/list-users.phtml
        $dbname = APPLICATION_PATH . '/../data/test.sqlite';
        $db = new Zend_Db_Adapter_Pdo_Sqlite(array('dbname' => $dbname));
        $stmt = $db->prepare('select * from users');
        var_dump(get_class($stmt));
        $stmt->execute();
        $users = $stmt->fetchAll();
        $this->view->users = $users;
    }

    public function brokenAction()
    {
        // This action is intentionally broken to test error handling
        $db = new Ibm(array('dbname' => 'testdb', 'username' => 'testuser', 'password' => 'testpass'));
        $stmt = $db->prepare('select * from users');
    }
}