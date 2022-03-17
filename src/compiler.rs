use crate::{code::{Symbol, Relocatable}, ast::Function};
use std::{fmt::Debug, collections::HashMap, cell::{RefCell, Cell}, rc::Rc, env::VarError};


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Type {
    Integer,
    Boolean,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Location {
    Local {
        stack_offset: usize,
    },
    Static {
        symbol: Symbol,
        atomic: bool,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Variable {
    pub(crate) name: Symbol,
    pub(crate) r#type: Type,
    pub(crate) location: Location,
}

pub(crate) trait Register : Copy + Debug + Eq + Ord {
    type Clobber: IntoIterator<Item = Self> + Extend<Self>;
    fn usable_registers() -> Vec<Self>;
    fn load_from(self, from: &Variable) -> (Relocatable, Self::Clobber);
    fn store_to(self, into: &Variable) -> (Relocatable, Self::Clobber);
    
    fn copy_from(self, src: Self) -> (Relocatable, Self::Clobber);
    fn copy_into(self, dst: Self) -> (Relocatable, Self::Clobber) {
        dst.copy_from(self)
    }

    fn add_assign(self, rhs: Self) -> (Relocatable, Self::Clobber);
    fn checked_add_assign(self, rhs: Self) -> (Relocatable, Self::Clobber);

    fn add(self, rhs: Self, result: Self) -> (Relocatable, Self::Clobber) {
        if self == result {
            return self.checked_add_assign(rhs);
        }
        let (add, mut clobbers) = self.add_assign(rhs);
        let (copy_into, copy_clobbers) = self.copy_into(result);
        clobbers.extend(copy_clobbers);
        clobbers.extend([self]);

        (add + copy_into, clobbers)
    }
    fn checked_add(self, rhs: Self, result: Self) -> (Relocatable, Self::Clobber) {
        if self == result {
            return self.checked_add_assign(rhs);
        }
        let (add, mut clobbers) = self.checked_add_assign(rhs);
        let (copy_into, copy_clobbers) = self.copy_into(result);
        clobbers.extend(copy_clobbers);
        clobbers.extend([self]);

        (add + copy_into, clobbers)
    }

    fn sub_assign(self, rhs: Self) -> (Relocatable, Self::Clobber);
    fn checked_sub_assign(self, rhs: Self) -> (Relocatable, Self::Clobber);

    fn sub(self, rhs: Self, result: Self) -> (Relocatable, Self::Clobber) {
        if self == result {
            return self.checked_sub_assign(rhs);
        }
        let (sub, mut clobbers) = self.sub_assign(rhs);
        let (copy_into, copy_clobbers) = self.copy_into(result);
        clobbers.extend(copy_clobbers);
        clobbers.extend([self]);

        (sub + copy_into, clobbers)
    }
    fn checked_sub(self, rhs: Self, result: Self) -> (Relocatable, Self::Clobber) {
        if self == result {
            return self.checked_sub_assign(rhs);
        }
        let (sub, mut clobbers) = self.checked_sub_assign(rhs);
        let (copy_into, copy_clobbers) = self.copy_into(result);
        clobbers.extend(copy_clobbers);
        clobbers.extend([self]);

        (sub + copy_into, clobbers)
    }
}

mod arch;

#[derive(Debug, Clone, Default)]
pub(crate) struct GlobalState {
    pub(crate) symbols: HashMap<Symbol, Location>,
}

pub enum CompileError {
    Todo,
}

trait Compilable<'ast, State = GlobalState> {
    type Output: Debug + Clone; // = ();
    fn compile(&'ast self, state: &mut State) -> Result<Self::Output, CompileError>;
}

pub fn compile(ast: &crate::ast::Module) -> Result<Relocatable, CompileError> {
    let mut state = GlobalState::default();
    ast.compile(&mut state)
}

impl<'ast> Compilable<'ast> for crate::ast::Module {
    type Output = Relocatable;
    fn compile(&'ast self, state: &mut GlobalState) -> Result<Self::Output, CompileError> {
        self.items
            .iter()
            .map(|item| item.compile(state))
            .reduce(|a, b| Ok(a? + b?))
            .unwrap_or_else(|| Ok(Default::default()))
    }
}

impl<'ast> Compilable<'ast> for crate::ast::Item {
    type Output = Relocatable;
    fn compile(&'ast self, state: &mut GlobalState) -> Result<Self::Output, CompileError> {
        match self {
            crate::ast::Item::FunctionItem(item) => item.compile(state),
            crate::ast::Item::StaticItem(item) => item.compile(state),
        }
    }
}

struct FunctionScopeState<'parent, 'ast> {
    global_state: &'parent GlobalState,
    parent_state: Option<&'parent FunctionScopeState<'parent, 'ast>>,
    stack_depth: usize,
    locals: HashMap<&'ast str, Rc<Variable>>,
}

impl<'a, 'b> FunctionScopeState<'a, 'b> {
    fn new_stack_slot(&mut self) -> usize {
        let slot = self.stack_depth;
        self.stack_depth += 1;
        slot
    }
    fn new_local(&mut self, name: &'b str) -> Rc<Variable> {
        let slot = self.new_stack_slot();
        let sym = Symbol::new_local();
        let var = Variable{
            name: sym.clone(),
            r#type: Type::Integer,
            location: Location::Local { stack_offset: slot },
        };
        let var = Rc::new(var);
        eprintln!("TODO: handle shadowing?");
        self.locals.insert(name, Rc::clone(&var));
        var
    }
}

impl<'ast> Compilable<'ast> for crate::ast::Function {
    type Output = Relocatable;
    fn compile(&'ast self, state: &mut GlobalState) -> Result<Self::Output, CompileError> {
        let mut state = FunctionScopeState {
            global_state: state,
            parent_state: None,
            stack_depth: 0,
            locals: Default::default(),
        };
        let parameter_variables = self.parameters
            .iter()
            .map(|parameter| {
                let local_var = state.new_local(&parameter.name);
                (parameter, local_var)
            })
            .collect::<Vec<_>>()
            ;
        let body = self.body.compile(&mut state)?;
            
        todo!("prologue and epilogue")
    }
}

impl<'ast> Compilable<'ast> for crate::ast::Static {
    type Output = Relocatable;
    fn compile(&'ast self, state: &mut GlobalState) -> Result<Self::Output, CompileError> {
        todo!()
    }
}

impl<'a, 'ast> Compilable<'ast, FunctionScopeState<'a, 'ast>> for crate::ast::Block {
    type Output = Relocatable;
    fn compile(&'ast self, state: &mut FunctionScopeState<'a, 'ast>) -> Result<Self::Output, CompileError> {
        self.statements
            .iter()
            .map(|item| item.compile(state))
            .reduce(|a, b| Ok(a? + b?))
            .unwrap_or_else(|| Ok(Default::default()))
    }
}

impl<'a, 'ast> Compilable<'ast, FunctionScopeState<'a, 'ast>> for crate::ast::Statement {
    type Output = Relocatable;
    fn compile(&'ast self, state: &mut FunctionScopeState<'a, 'ast>) -> Result<Self::Output, CompileError> {
        match self {
            crate::ast::Statement::LetStatement(stmt) => stmt.compile(state),
            crate::ast::Statement::ExpressionStatement(expr) => Ok(expr.compile(state)?.0),
            crate::ast::Statement::LoopStatement(stmt) => stmt.compile(state),
            crate::ast::Statement::ReturnStatement(expr) => {
                todo!("compile return statement")
            },
        }
    }
}

impl<'a, 'ast> Compilable<'ast, FunctionScopeState<'a, 'ast>> for crate::ast::Let {
    type Output = Relocatable;
    fn compile(&'ast self, state: &mut FunctionScopeState<'a, 'ast>) -> Result<Self::Output, CompileError> {
        todo!()
    }
}

impl<'a, 'ast> Compilable<'ast, FunctionScopeState<'a, 'ast>> for crate::ast::Loop {
    type Output = Relocatable;
    fn compile(&'ast self, state: &mut FunctionScopeState<'a, 'ast>) -> Result<Self::Output, CompileError> {
        todo!()
    }
}

impl<'a, 'ast> Compilable<'ast, FunctionScopeState<'a, 'ast>> for crate::ast::Expression {
    // Returns the temporary that was created for the value of this expression.
    type Output = (Relocatable, Rc<Variable>);
    fn compile(&'ast self, state: &mut FunctionScopeState<'a, 'ast>) -> Result<Self::Output, CompileError> {
        todo!()
    }
}
