//! Operators used by jinko's BinaryOp struct. This module is not public, and is only
//! used by the BinaryOp structure.

use std::convert::From;

/// All the binary operators available
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    LeftParenthesis,
    RightParenthesis,
    Equals,
    NotEquals
}

impl From<&str> for Operator {
    fn from(op_str: &str) -> Operator {
        match op_str {
            "+" => Operator::Add,
            "-" => Operator::Sub,
            "*" => Operator::Mul,
            "/" => Operator::Div,
            "(" => Operator::LeftParenthesis,
            ")" => Operator::RightParenthesis,
            "==" => Operator::Equals,
            "!=" => Operator::NotEquals,
            _ => unreachable!("Invalid operator: {}", op_str),
        }
    }
}

impl From<Operator> for &str {
    fn from(op: Operator) -> &'static str {
        match op {
            Operator::Add => "+",
            Operator::Sub => "-",
            Operator::Mul => "*",
            Operator::Div => "/",
            Operator::LeftParenthesis => "(",
            Operator::RightParenthesis => ")",
            Operator::Equals => "==",
            Operator::NotEquals => "==",
        }
    }
}

impl Operator {
    /// Return the operator's precedence according to the Shunting Yard algorithm
    pub fn precedence(&self) -> u8 {
        match self {
            // Classic SY operator precedence
            Operator::Mul | Operator::Div => 3,
            Operator::Add | Operator::Sub => 2,
            Operator::Equals | Operator::NotEquals => 0,

            // Special operators. They don't really have a precedence value, and it's
            // never used
            Operator::LeftParenthesis | Operator::RightParenthesis => 0,
        }
    }

    /// Is the operator a left associative one
    pub fn is_left_associative(&self) -> bool {
        // FIXME: Not entirely true
        // - Changes once we add more operators such as the Power one
        // match self {
        //     _ => true,
        // }

        true
    }
}
