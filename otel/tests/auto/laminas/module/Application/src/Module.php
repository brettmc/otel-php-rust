<?php

declare(strict_types=1);

namespace Application;

use Laminas\Mvc\MvcEvent;
use OpenTelemetry\API\Trace\LocalRootSpan;

class Module
{
    public function getConfig(): array
    {
        /** @var array $config */
        $config = include __DIR__ . '/../config/module.config.php';
        return $config;
    }


    public function onBootstrap($e)
    {
        $eventManager = $e->getApplication()->getEventManager();
        $eventManager->attach(
            MvcEvent::EVENT_ROUTE,
            [$this, 'onRoutePost'],
            -100 // Negative priority: runs after routing
        );
    }

    public function onRoutePost(MvcEvent $e)
    {
        //var_dump($_SERVER);
    }
}
