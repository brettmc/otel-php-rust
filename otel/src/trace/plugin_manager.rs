use super::plugin::Plugin;
use crate::trace::plugins::{
    test::TestPlugin,
    roadrunner::RoadRunnerPlugin,
};
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin + Send + Sync>>,
}

impl PluginManager {
    pub fn new() -> Self {
        let mut manager = Self {plugins: vec![] };
        manager.init();
        manager
    }

    fn init(&mut self) {
        #[cfg(feature="test")]
        self.plugins.push(Box::new(TestPlugin));
        self.plugins.push(Box::new(RoadRunnerPlugin));
    }

    pub fn plugins(&self) -> &Vec<Box<dyn Plugin + Send + Sync>> {
        &self.plugins
    }
}
