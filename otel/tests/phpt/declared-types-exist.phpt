--TEST--
Every declared parameter and return type names a class or interface that exists
--EXTENSIONS--
otel
--INI--
otel.cli.enabled=1
--ENV--
OTEL_TRACES_EXPORTER=none
--FILE--
<?php
// PHP resolves parameter and return types lazily, so a misspelled class name in a type
// hint is invisible at runtime: the method works, but reflection reports a type that
// does not exist and userland cannot type-check against the signature. A real typo of
// exactly this kind (OpenTelemetry\API\Span\SpanInterface, a namespace that has never
// existed) shipped undetected in LocalRootSpan::current().
//
// Rather than pin each signature individually, walk everything the extension registers
// and assert that every declared type resolves. The FQNs now come from shared consts,
// which stops two spellings of the same name from drifting apart — but nothing stops a
// new hint from being written as a fresh literal, and this catches that too.

$classes = array_merge(
    get_declared_classes(),
    get_declared_interfaces(),
);
$errors = [];
$checked = 0;

foreach ($classes as $class) {
    if (!str_starts_with($class, 'OpenTelemetry\\')) {
        continue;
    }
    $rc = new ReflectionClass($class);
    foreach ($rc->getMethods() as $method) {
        if ($method->getDeclaringClass()->getName() !== $class) {
            continue; // inherited: reported against the class that declares it
        }
        $types = [];
        if ($method->hasReturnType()) {
            $types['return'] = $method->getReturnType();
        }
        foreach ($method->getParameters() as $param) {
            if ($param->hasType()) {
                $types['$' . $param->getName()] = $param->getType();
            }
        }
        foreach ($types as $position => $type) {
            foreach (($type instanceof ReflectionUnionType ? $type->getTypes() : [$type]) as $single) {
                if ($single->isBuiltin()) {
                    continue;
                }
                $name = $single->getName();
                $checked++;
                if (!class_exists($name) && !interface_exists($name)) {
                    $errors[] = sprintf('%s::%s(): %s declares unknown type %s',
                        $class, $method->getName(), $position, $name);
                }
            }
        }
    }
}

// A zero count would make this test pass by checking nothing.
var_dump($checked > 0);
var_dump($errors);
?>
--EXPECT--
bool(true)
array(0) {
}
