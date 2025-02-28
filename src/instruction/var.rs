//! Represents the usage of a variable, for example when returning from
//! a block. In jinko, variables cannot be uninitialized. Therefore, there is no
//! need to keep an option of an instance. A variable is either there, fully initialized,
//! or it's not.

use crate::instruction::TypeDec;
use crate::log;
use crate::typechecker::{CheckedType, TypeCtx};
use crate::Generic;
use crate::{Context, ErrKind, Error, InstrKind, Instruction, ObjectInstance, TypeCheck};

#[derive(Clone)]
pub struct Var {
    name: String,
    mutable: bool,
    instance: ObjectInstance,
    cached_type: Option<CheckedType>,
}

impl Var {
    /// Create a new variable usage with the given name
    pub fn new(name: String) -> Var {
        Var {
            name,
            mutable: false,
            instance: ObjectInstance::empty(),
            cached_type: None,
        }
    }

    /// Return the name of the variable
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return a copy of the variable's instance
    pub fn instance(&self) -> ObjectInstance {
        self.instance.clone()
    }

    /// Is a variable mutable or not
    pub fn mutable(&self) -> bool {
        self.mutable
    }

    /// Set the instance contained in a variable
    pub fn set_instance(&mut self, instance: ObjectInstance) {
        self.instance = instance;
    }

    /// Change the mutability of a variable
    pub fn set_mutable(&mut self, mutable: bool) {
        self.mutable = mutable;
    }

    pub fn set_type(&mut self, ty: TypeDec) {
        self.instance.set_ty(CheckedType::Resolved(ty.into()))
    }
}

impl Instruction for Var {
    fn kind(&self) -> InstrKind {
        InstrKind::Expression(None)
    }

    fn print(&self) -> String {
        let mut base = self.name.clone();
        if let CheckedType::Resolved(ty) = self.instance.ty() {
            base = format!("{} /* : {} */", base, ty.id());
        }

        format!("{} = {}", base, self.instance.as_string())
    }

    fn execute(&self, ctx: &mut Context) -> Option<ObjectInstance> {
        let var = match ctx.get_variable(self.name()) {
            Some(v) => v,
            None => {
                ctx.error(
                    Error::new(ErrKind::Context)
                        .with_msg(format!("variable has not been declared: {}", self.name)),
                );

                return None;
            }
        };

        log!("var: {}", var.print());

        Some(var.instance())
    }
}

impl TypeCheck for Var {
    fn resolve_type(&mut self, ctx: &mut TypeCtx) -> CheckedType {
        match ctx.get_var(self.name()) {
            Some(var_ty) => var_ty.clone(),
            None => {
                ctx.error(
                    Error::new(ErrKind::TypeChecker)
                        .with_msg(format!("use of undeclared variable: `{}`", self.name())),
                );
                CheckedType::Error
            }
        }
    }

    fn set_cached_type(&mut self, ty: CheckedType) {
        self.cached_type = Some(ty)
    }

    fn cached_type(&self) -> Option<&CheckedType> {
        self.cached_type.as_ref()
    }
}

impl Default for Var {
    fn default() -> Self {
        Var::new(String::new())
    }
}

impl Generic for Var {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::JkInt;
    use crate::ToObjectInstance;
    use crate::{jinko, jinko_fail};

    #[test]
    fn keep_instance() {
        let mut i = Context::new();
        let mut v = Var::new("a".to_string());

        let instance = JkInt::from(15).to_instance();
        v.set_instance(instance.clone());

        i.add_variable(v.clone()).unwrap();

        assert_eq!(v.execute(&mut i).unwrap(), instance);
    }

    #[test]
    fn tc_valid() {
        jinko! {
            a = 15;
            a
        };
    }

    #[test]
    fn tc_invalid() {
        jinko_fail! {
            // undeclared variable
            a
        };
    }
}
