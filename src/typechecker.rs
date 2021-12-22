//! The [`TypeCheck`] trait enables a value to get resolved to a type. If the value will
//! need to get its type checked multiple times, then it can implement the [`CachedTypeCheck`]
//! trait on top of it.

use crate::{error::ErrorHandler, instruction::TypeId, Error, ScopeMap};
use colored::Colorize;
use std::{
    collections::HashSet,
    fmt::{Display, Formatter, Result as FmtResult},
    path::{Path, PathBuf},
};

/// The [`CheckedType`] enum contains three possible states about the type. Either the
/// type has been properly resolved to something, or it corresponds to a Void type. If the
/// type has not been resolved yet, it can be unknown.
#[derive(Clone, PartialEq, Debug)]
pub enum CheckedType {
    Resolved(TypeId),
    Void,
    Unknown,
}

impl Default for CheckedType {
    fn default() -> CheckedType {
        CheckedType::Unknown
    }
}

impl Display for CheckedType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let ty_str = match self {
            CheckedType::Resolved(ty) => ty.id(),
            CheckedType::Void => "void",
            CheckedType::Unknown => "!!unknown!!",
        };

        write!(f, "{}", ty_str.purple())
    }
}

struct FunctionType {
    /// We need to keep track of the function's arguments name and expected type
    args_ty: Vec<(String, CheckedType)>,
    return_ty: CheckedType,
}

struct CustomTypeType {
    self_ty: CheckedType,
    fields_ty: Vec<(String, CheckedType)>,
}

// TODO: Should we factor this into a `Context` trait? All contexts will share some
// similarities, such as the ability to emit errors or enter and exit scopes
/// The [`TypeCtx`]'s role is to keep track of declared types based on the scope they
/// were created in. In doing so, it allows the [`Instruction`]s to perform a type lookup
/// in order to resolve to a concrete type.Each declaration (First [`VarAssign`],
/// [`FunctionDec`]s and [`TypeDec`]s) can also declare a new type and make it available
/// to all instructions in the avaialble scopes.
pub struct TypeCtx {
    /// Reference to the context's error handler
    pub(crate) error_handler: ErrorHandler,
    /// The Type Context stores [`CheckedType`]s for all three generics kept in the scope
    /// map: Variables, Functions and Types.
    /// For functions, we keep a vector of argument types as well as the return type.
    /// Custom types need to keep a type for themselves, as well as types for all their fields
    types: ScopeMap<CheckedType, FunctionType, CustomTypeType>,
    /// Is the type context executing its second pass or not. The second pass of the typechecking
    /// process is to resolve generic calls and make sure that the functions called on the
    /// expanded types are actually present. Plus, this also allows us to call/instantiate
    /// functions/types defined after the call/instantiation.
    /// At this stage, we do not emit "already defined" errors, as they will all have been
    /// emitted by the first pass already.
    is_second_pass: bool,
    // FIXME: Remove both of these fields...
    /// Path from which the typechecking context was instantiated
    path: Option<PathBuf>,
    included: HashSet<PathBuf>,
}

impl TypeCtx {
    /// Create a new empty [`TypeCtx`]
    pub fn new() -> TypeCtx {
        // FIXME: This doesn't contain builtins
        let mut ctx = TypeCtx {
            error_handler: ErrorHandler::default(),
            types: ScopeMap::new(),
            is_second_pass: false,
            path: None,
            included: HashSet::new(),
        };

        macro_rules! declare_primitive {
            ($ty_name:ident) => {
                ctx.declare_custom_type(
                    String::from(stringify!($ty_name)),
                    CheckedType::Resolved(TypeId::from(stringify!($ty_name))),
                    vec![],
                )
                .unwrap();
            };
        }

        ctx.scope_enter();

        declare_primitive!(bool);
        declare_primitive!(int);
        declare_primitive!(float);
        declare_primitive!(char);
        declare_primitive!(string);

        ctx
    }

    pub fn start_second_pass(&mut self) {
        self.is_second_pass = true
    }

    pub fn is_second_pass(&self) -> bool {
        self.is_second_pass
    }

    // FIXME: Remove these three functions
    pub fn set_path(&mut self, path: Option<PathBuf>) {
        self.path = path
    }

    pub fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    pub fn include(&mut self, path: PathBuf) {
        self.included.insert(path);
    }

    pub fn is_included(&self, path: &Path) -> bool {
        self.included.contains(path)
    }

    /// Enter a new scope. This is the same as lexical scopes
    pub fn scope_enter(&mut self) {
        self.types.scope_enter()
    }

    /// Exit a previously created scope. This is the same as lexical scopes
    pub fn scope_exit(&mut self) {
        self.types.scope_exit()
    }

    /// Declare a newly-created variable's type
    pub fn declare_var(&mut self, name: String, ty: CheckedType) -> Result<(), Error> {
        self.types
            .add_variable(name, ty)
            .or_else(|e| if self.is_second_pass { Ok(()) } else { Err(e) })
    }

    /// Declare a newly-created function's type
    pub fn declare_function(
        &mut self,
        name: String,
        args_ty: Vec<(String, CheckedType)>,
        return_ty: CheckedType,
    ) -> Result<(), Error> {
        self.types
            .add_function(name, FunctionType { args_ty, return_ty })
            .or_else(|e| if self.is_second_pass { Ok(()) } else { Err(e) })
    }

    /// Declare a newly-created custom type
    pub fn declare_custom_type(
        &mut self,
        name: String,
        self_ty: CheckedType,
        fields_ty: Vec<(String, CheckedType)>,
    ) -> Result<(), Error> {
        self.types
            .add_type(name, CustomTypeType { self_ty, fields_ty })
            .or_else(|e| if self.is_second_pass { Ok(()) } else { Err(e) })
    }

    /// Access a previously declared variable's type
    pub fn get_var(&mut self, name: &str) -> Option<&CheckedType> {
        self.types.get_variable(name)
    }

    /// Access a previously declared function's type
    pub fn get_function(
        &mut self,
        name: &str,
    ) -> Option<(&Vec<(String, CheckedType)>, &CheckedType)> {
        self.types
            .get_function(name)
            .map(|func| (&func.args_ty, &func.return_ty))
    }

    /// Access a previously declared custom type
    pub fn get_custom_type(
        &mut self,
        name: &str,
    ) -> Option<(&CheckedType, &Vec<(String, CheckedType)>)> {
        self.types
            .get_type(name)
            .map(|custom_type| (&custom_type.self_ty, &custom_type.fields_ty))
    }

    /// Create a new error to propagate to the original context
    pub fn error(&mut self, err: Error) {
        self.error_handler.add(err)
    }
}

impl Default for TypeCtx {
    fn default() -> TypeCtx {
        TypeCtx::new()
    }
}

/// The [`TypeCheck`] trait allows an [`Instruction`] to see its type resolved statically.
/// There are three possible return values:
///     - Resolve(type): This means that the type of the [`Instruction`] was abled to
///     get resolved statically
///     - Void: The [`Instruction`] is not of any type. It corresponds to a statement.
///     - Unknown: This means that even after the typechecking pass, the instruction's
///     type is still unclear.
pub trait TypeCheck {
    /// Go through the context in order to figure out the type of an instruction.
    /// This function should report errors using the context, and the [`ErrKind::TypeCheck`]
    /// error kind.
    fn resolve_type(&self, ctx: &mut TypeCtx) -> CheckedType;
}

/// Some [`Instruction`]s need to have their type checked multiple times. For example, a
/// function might be called in multiple places, by various vaiables. These instructions
/// can "cache" their type in order to not go through the resolver each time
pub trait CachedTypeCheck: TypeCheck {
    /// Store the given type somewhere in order to cache it
    fn set_type(&mut self, ty: CheckedType);

    /// Return a reference to the cached type, previously stored using [`set_type()`]
    fn get_type(&self) -> &CheckedType;

    /// If the type is not known yet, compute it by going through the [`TypeCheck`]
    /// resolver. Otherwise, fetch it from the cached instance
    fn type_check(&mut self, ctx: &mut TypeCtx) {
        if let CheckedType::Unknown = self.get_type() {
            self.set_type(self.resolve_type(ctx))
        }
    }
}
