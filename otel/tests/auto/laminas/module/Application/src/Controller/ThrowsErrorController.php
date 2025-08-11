<?php

declare(strict_types=1);

namespace Application\Controller;

use Laminas\Mvc\Controller\AbstractActionController;

class ThrowsErrorController extends AbstractActionController
{
    public function boomAction()
    {
        throw new \RuntimeException('boom');
    }
}
