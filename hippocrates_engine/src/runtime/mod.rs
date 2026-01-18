pub mod environment;
pub mod evaluator;
mod evaluator_tests;
pub mod executor;
pub mod scheduler;
pub mod validator;

pub use environment::Environment;
pub use evaluator::Evaluator;
pub use executor::ExecutionMode;
pub use executor::Executor;

use crate::ast::Plan;

pub struct Engine {
    pub env: Environment,
    pub mode: ExecutionMode,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            env: Environment::new(),
            mode: ExecutionMode::RealTime,
        }
    }

    pub fn set_mode(&mut self, mode: ExecutionMode) {
        self.mode = mode;
    }

    pub fn load_plan(&mut self, plan: Plan) {
        self.env.load_plan(plan);
    }

    pub fn execute(&mut self, plan_name: &str) {
        let mut executor = Executor::new();
        executor.set_mode(self.mode);
        executor.execute_plan(&mut self.env, plan_name);
    }

    // Helper for testing
    pub fn set_value(&mut self, name: &str, val: crate::domain::RuntimeValue) {
        self.env.set_value(name, val);
    }
}
