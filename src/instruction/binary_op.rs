//! Binary operations apply an operation on two Instructions. When writing
//! 1 + 2, a BinaryOp will be created containing "1" as a left hand side operand, "2" as
//! a right hand side operand and "+" as the operator.
//!
//! The available operators are `+`, `-`, `*` and `/`.
//! That is `Add`, `Substract`, `Multiply` and `Divide`.

use crate::{
    instruction::Operator,
    typechecker::{CheckedType, TypeCtx},
    Context, ErrKind, Error, FromObjectInstance, InstrKind, Instruction, JkFloat, JkInt,
    ObjectInstance, TypeCheck, Value,
};

/// The `BinaryOp` struct contains two expressions and an operator, which can be an arithmetic
/// or a comparison one
#[derive(Clone)]
pub struct BinaryOp {
    lhs: Box<dyn Instruction>,
    rhs: Box<dyn Instruction>,
    op: Operator,
}

impl BinaryOp {
    /// Create a new `BinaryOp` from two instructions and an operator
    pub fn new(lhs: Box<dyn Instruction>, rhs: Box<dyn Instruction>, op: Operator) -> Self {
        BinaryOp { lhs, rhs, op }
    }

    /// Return the operator used by the BinaryOp
    #[cfg(test)]
    pub fn operator(&self) -> Operator {
        self.op
    }

    // Get a reference on the left side member of a BinaryOp
    #[cfg(test)]
    pub fn lhs(&self) -> &Box<dyn Instruction> {
        &self.lhs
    }

    /// Get a reference on the right side member of a BinaryOp
    #[cfg(test)]
    pub fn rhs(&self) -> &Box<dyn Instruction> {
        &self.rhs
    }

    // FIXME: Use Context::execute_expression
    /// Execute a node of the binary operation
    fn execute_node(&self, node: &dyn Instruction, ctx: &mut Context) -> Option<ObjectInstance> {
        match node.execute(ctx) {
            None => {
                ctx.error(Error::new(ErrKind::Context).with_msg(format!(
                    "invalid use of statement in binary operation: {}",
                    node.print()
                )));
                None
            }
            Some(v) => Some(v),
        }
    }
}

impl Instruction for BinaryOp {
    fn kind(&self) -> InstrKind {
        InstrKind::Expression(None)
    }

    fn print(&self) -> String {
        format!(
            "{} {} {}",
            self.lhs.print(),
            self.op.as_str(),
            self.rhs.print()
        )
    }

    fn execute(&self, ctx: &mut Context) -> Option<ObjectInstance> {
        ctx.debug_step("BINOP ENTER");

        ctx.debug("OP", self.op.as_str());

        let l_value = self.execute_node(&*self.lhs, ctx)?;
        let r_value = self.execute_node(&*self.rhs, ctx)?;

        // FIXME: This produces unhelpful errors for now
        if l_value.ty() != r_value.ty() {
            return None;
        }

        let return_value;

        // At this point, we will already have checked whether or not a binary op
        // is valid type-wise. So we can unwrap at will. If a type is still unknown
        // at this point, this is an interpreter error
        match l_value.ty() {
            CheckedType::Resolved(ty) => match ty.id() {
                "int" => {
                    let res = JkInt::from_instance(&l_value)
                        .do_op(&JkInt::from_instance(&r_value), self.op);
                    return_value = match res {
                        Ok(r) => r,
                        Err(e) => {
                            ctx.error(e);
                            return None;
                        }
                    };
                }
                "float" => {
                    let res = JkFloat::from_instance(&l_value)
                        .do_op(&JkFloat::from_instance(&r_value), self.op);
                    return_value = match res {
                        Ok(r) => r,
                        Err(e) => {
                            ctx.error(e);
                            return None;
                        }
                    };
                }
                _ => unreachable!(
                    "attempting binary operation with void type or unknown type AFTER typechecking"
                ),
            },
            _ => unreachable!(
                "attempting binary operation with void type or unknown type AFTER typechecking"
            ),
        }

        // // FIXME: DISGUSTING and do not unwap
        // match l_value.ty().unwrap().name() {
        //     // FIXME: Absolutely DISGUSTING
        //     "int" => {
        //         let res =
        //             JkInt::from_instance(&l_value).do_op(&JkInt::from_instance(&r_value), self.op);
        //         return_value = match res {
        //             Ok(r) => r,
        //             Err(e) => {
        //                 ctx.error(e);
        //                 return None;
        //             }
        //         };
        //     }

        //     "float" => {
        //         let res = JkFloat::from_instance(&l_value)
        //             .do_op(&JkFloat::from_instance(&r_value), self.op);
        //         return_value = match res {
        //             Ok(r) => r,
        //             Err(e) => {
        //                 ctx.error(e);
        //                 return None;
        //             }
        //         }
        //     }
        //     _ => todo!("Implement empty types?"),
        // }

        ctx.debug_step("BINOP EXIT");

        Some(return_value)
    }
}

impl TypeCheck for BinaryOp {
    fn resolve_type(&self, ctx: &mut TypeCtx) -> CheckedType {
        let l_type = self.lhs.resolve_type(ctx);
        let r_type = self.rhs.resolve_type(ctx);

        if l_type != r_type {
            ctx.error(Error::new(ErrKind::TypeChecker).with_msg(format!(
                "trying to do binary operation on invalid types: {} {} {}",
                l_type,
                self.op.as_str(),
                r_type,
            )));
            return CheckedType::Unknown;
        }

        l_type
    }
}

// TODO: Add typechecking tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::JkInt;
    use crate::Context;
    use crate::ToObjectInstance;
    use crate::{jinko, jinko_fail};

    fn binop_assert(l_num: i64, r_num: i64, op_string: &str, res: i64) {
        let l = Box::new(JkInt::from(l_num));
        let r = Box::new(JkInt::from(r_num));
        let op = Operator::new(op_string);

        let binop = BinaryOp::new(l, r, op);

        let mut i = Context::new();

        assert_eq!(
            binop.execute(&mut i).unwrap(),
            JkInt::from(res).to_instance(),
        );
        assert!(!i.error_handler.has_errors());
    }

    #[test]
    fn t_binop_add_same() {
        binop_assert(12, 12, "+", 24);
    }

    #[test]
    fn t_binop_add_l_diff() {
        binop_assert(12, 2, "+", 14);
    }

    #[test]
    fn t_binop_add_r_diff() {
        binop_assert(2, 99, "+", 101);
    }

    #[test]
    fn t_binop_mul_same() {
        binop_assert(12, 12, "*", 144);
    }

    #[test]
    fn t_binop_mul_l_diff() {
        binop_assert(12, 2, "*", 24);
    }

    #[test]
    fn t_binop_mul_r_diff() {
        binop_assert(2, 99, "*", 198);
    }

    #[test]
    fn t_binop_rhs_execute() {
        let r_bin = BinaryOp::new(
            Box::new(JkInt::from(12)),
            Box::new(JkInt::from(3)),
            Operator::new("*"),
        );
        let binary_op = BinaryOp::new(
            Box::new(JkInt::from(9)),
            Box::new(r_bin),
            Operator::new("-"),
        );

        let mut i = Context::new();

        assert_eq!(
            binary_op.rhs().execute(&mut i).unwrap(),
            JkInt::from(36).to_instance(),
        );
        assert!(!i.error_handler.has_errors());
    }

    #[test]
    fn t_binop_lhs_execute() {
        let l_bin = BinaryOp::new(
            Box::new(JkInt::from(12)),
            Box::new(JkInt::from(3)),
            Operator::new("*"),
        );
        let binary_op = BinaryOp::new(
            Box::new(l_bin),
            Box::new(JkInt::from(9)),
            Operator::new("-"),
        );

        let mut i = Context::new();

        assert_eq!(binary_op.operator(), Operator::Sub);

        assert_eq!(
            binary_op.lhs().execute(&mut i).unwrap(),
            JkInt::from(36).to_instance()
        );
        assert!(!i.error_handler.has_errors());
    }

    #[test]
    fn tc_binop_valid() {
        jinko! {
            t0 = 1 + 1;
            t2 = 1.0 + 1.4;
        };
    }

    #[test]
    fn tc_binop_from_func() {
        jinko! {
            func id(x: int) -> int { x }
            t0 = id(1) + id(id(id(id(14))));
        };
    }

    #[test]
    fn tc_binop_mismatched_valid() {
        jinko_fail! {
            t0 = 1 + '4';
            t2 = 1.0 + "hey";
        };
    }
}
