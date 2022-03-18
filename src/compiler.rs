use crate::{code::{Symbol, Relocatable, Object}, ast::Function};
use std::{fmt::Debug, collections::HashMap, cell::{RefCell, Cell}, rc::Rc, env::VarError, borrow::Cow};


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Type {
    Integer,
    Boolean,
    Function,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Location {
    Local {
        stack_index: usize,
    },
    Static {
        symbol: Symbol,
        atomic: bool,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Variable {
    pub(crate) name: Symbol,
    pub(crate) mutable: bool,
    pub(crate) r#type: Type,
    pub(crate) location: Location,
}


pub(crate) trait Machine : Debug + Copy {
    type Register: Copy + Debug + Eq + Ord;
    type Clobber: IntoIterator<Item = Self::Register> + Extend<Self::Register>;

    #[must_use]
    fn function_prologue_epilogue_abort(self, stack_slots: usize, arg_slots: Vec<usize>) -> Result<(Object, Object, Object), CompileError<'static>>;

    #[must_use]
    fn add_data(self, data: Vec<u8>, symbol: Symbol) -> Object;

    fn usable_registers(self) -> Vec<Self::Register>;
    #[must_use]
    fn load_from(self, into: Self::Register, from: &Variable) -> (Object, Self::Clobber);
    #[must_use]
    fn store_to(self, from: Self::Register, into: &Variable) -> (Object, Self::Clobber);
    
    #[must_use]
    fn copy_from(self, dst: Self::Register, src: Self::Register) -> (Object, Self::Clobber);
    #[must_use]
    fn copy_into(self, src: Self::Register, dst: Self::Register) -> (Object, Self::Clobber) {
        self.copy_from(dst, src)
    }

    #[must_use]
    fn add_assign(self, lhs: Self::Register, rhs: Self::Register) -> (Object, Self::Clobber);
    #[must_use]
    fn checked_add_assign(self, lhs: Self::Register, rhs: Self::Register) -> (Object, Self::Clobber);

    #[must_use]
    fn add(self, lhs: Self::Register, rhs: Self::Register, result: Self::Register) -> (Object, Self::Clobber) {
        if lhs == result {
            return self.checked_add_assign(lhs, rhs);
        }
        let (add, mut clobbers) = self.add_assign(lhs, rhs);
        let (copy_into, copy_clobbers) = self.copy_into(lhs, result);
        clobbers.extend(copy_clobbers);
        clobbers.extend([lhs]);

        (add + copy_into, clobbers)
    }
    #[must_use]
    fn checked_add(self, lhs: Self::Register, rhs: Self::Register, result: Self::Register) -> (Object, Self::Clobber) {
        if lhs == result {
            return self.checked_add_assign(lhs, rhs);
        }
        let (add, mut clobbers) = self.checked_add_assign(lhs, rhs);
        let (copy_into, copy_clobbers) = self.copy_into(lhs, result);
        clobbers.extend(copy_clobbers);
        clobbers.extend([lhs]);

        (add + copy_into, clobbers)
    }

    #[must_use]
    fn sub_assign(self, lhs: Self::Register, rhs: Self::Register) -> (Object, Self::Clobber);
    #[must_use]
    fn checked_sub_assign(self, lhs: Self::Register, rhs: Self::Register) -> (Object, Self::Clobber);

    #[must_use]
    fn sub(self, lhs: Self::Register, rhs: Self::Register, result: Self::Register) -> (Object, Self::Clobber) {
        if lhs == result {
            return self.checked_sub_assign(lhs, rhs);
        }
        let (sub, mut clobbers) = self.sub_assign(lhs, rhs);
        let (copy_into, copy_clobbers) = self.copy_into(lhs, result);
        clobbers.extend(copy_clobbers);
        clobbers.extend([lhs]);

        (sub + copy_into, clobbers)
    }
    #[must_use]
    fn checked_sub(self, lhs: Self::Register, rhs: Self::Register, result: Self::Register) -> (Object, Self::Clobber) {
        if lhs == result {
            return self.checked_sub_assign(lhs, rhs);
        }
        let (sub, mut clobbers) = self.checked_sub_assign(lhs, rhs);
        let (copy_into, copy_clobbers) = self.copy_into(lhs, result);
        clobbers.extend(copy_clobbers);
        clobbers.extend([lhs]);

        (sub + copy_into, clobbers)
    }
}

mod arch;

#[derive(Debug, Clone, Default)]
pub(crate) struct GlobalState {
    pub(crate) symbols: HashMap<Symbol, Rc<Variable>>,
}

impl GlobalState {
    fn new_static<M: Machine>(&mut self, machine: M, value: isize, symbol: Symbol) -> (Object, Rc<Variable>) {
        let var = Rc::new(Variable{
            name: symbol.clone(),
            mutable: false,
            r#type: Type::Integer,
            location: Location::Static { symbol: symbol.clone(), atomic: false },
        });
        self.symbols.insert(symbol.clone(), Rc::clone(&var));
        (machine.add_data(value.to_le_bytes().into(), symbol), var)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompileError<'ast> {
    UndefinedSymbol(&'ast str),
    Other(Cow<'static, str>),
}

pub(crate) trait Compilable<'ast, M: Machine, State = GlobalState> {
    type Output: Debug + Clone; // = ();
    fn compile(&'ast self, machine: M, state: &mut State) -> Result<Self::Output, CompileError<'ast>>;
}

pub fn compile(ast: &crate::ast::Module) -> Result<Object, CompileError<'_>> {
    let mut state = GlobalState::default();
    ast.compile(arch::Machine, &mut state)
}

pub(crate) fn compile_helper<'a, C: Compilable<'a, arch::Machine, Output = Object>>(ast: &'a C) -> Result<Object, CompileError<'_>> {
    let mut state = GlobalState::default();
    ast.compile(arch::Machine, &mut state)
}

impl<'ast, M: Machine> Compilable<'ast, M> for crate::ast::Module {
    type Output = Object;
    fn compile(&'ast self, machine: M, state: &mut GlobalState) -> Result<Self::Output, CompileError<'ast>> {
        self.items
            .iter()
            .map(|item| item.compile(machine, state))
            .reduce(|a, b| Ok(a? + b?))
            .unwrap_or_else(|| Ok(Default::default()))
    }
}

impl<'ast, M: Machine> Compilable<'ast, M> for crate::ast::Item {
    type Output = Object;
    fn compile(&'ast self, machine: M, state: &mut GlobalState) -> Result<Self::Output, CompileError<'ast>> {
        match self {
            crate::ast::Item::FunctionItem(item) => item.compile(machine, state),
            crate::ast::Item::StaticItem(item) => item.compile(machine, state),
        }
    }
}

struct FunctionScopeState<'parent, 'ast> {
    global_state: &'parent mut GlobalState,
    parent_state: Option<&'parent FunctionScopeState<'parent, 'ast>>,
    stack_depth: usize,
    locals: HashMap<&'ast str, Rc<Variable>>,
    function_return: Symbol,
    function_abort: Symbol,
}

impl<'parent, 'ast> FunctionScopeState<'parent, 'ast> {
    fn new(global_state: &'parent mut GlobalState) -> Self {
        let function_return = Symbol::new_local();
        let function_abort = Symbol::new_local();
        Self {
            global_state,
            parent_state: None,
            stack_depth: 0,
            locals: Default::default(),
            function_return,
            function_abort,
        }
    }
    fn new_stack_slot(&mut self) -> usize {
        let slot = self.stack_depth;
        self.stack_depth += 1;
        slot
    }
    fn new_local(&mut self, name: &'ast str, mutable: bool) -> Rc<Variable> {
        let slot = self.new_stack_slot();
        let sym = Symbol::new_local();
        let var = Variable{
            name: sym.clone(),
            mutable,
            r#type: Type::Integer,
            location: Location::Local { stack_index: slot },
        };
        let var = Rc::new(var);
        eprintln!("TODO: handle shadowing?");
        self.locals.insert(name, Rc::clone(&var));
        var
    }
    fn new_temporary(&mut self) -> Rc<Variable> {
        let slot = self.new_stack_slot();
        let sym = Symbol::new_local();
        let var = Variable{
            name: sym.clone(),
            mutable: false,
            r#type: Type::Integer,
            location: Location::Local { stack_index: slot },
        };
        let var = Rc::new(var);
        // eprintln!("TODO: handle shadowing?");
        // self.locals.insert(name, Rc::clone(&var));
        var
    }
    fn get_variable(&self, name: &str) -> Option<Rc<Variable>> {
        match self.locals.get(name) {
            Some(var) => Some(Rc::clone(var)),
            None => match self.parent_state {
                Some(state) => state.get_variable(name),
                None => {
                    let symbol = Symbol::new_global(name.into());
                    self.global_state.symbols.get(&symbol).map(Rc::clone)
                }
            }
        }
    }
}

impl<'ast, M: Machine> Compilable<'ast, M> for crate::ast::Function {
    type Output = Object;
    fn compile(&'ast self, machine: M, state: &mut GlobalState) -> Result<Self::Output, CompileError<'ast>> {
        let mut state = FunctionScopeState::new(state);
        let arg_slots = self.parameters
            .iter()
            .map(|parameter| {
                let local_var = state.new_local(&parameter.name, parameter.mutable);
                match local_var.location {
                    Location::Local { stack_index } => stack_index,
                    Location::Static { .. } => unreachable!(),
                }
            })
            .collect::<Vec<_>>()
            ;
        let body = self.body.compile(machine, &mut state)?;

        let (prologue, epilogue, abort) = machine.function_prologue_epilogue_abort(state.stack_depth, arg_slots)?;
            
        todo!("prologue and epilogue")
    }
}

impl<'ast, M: Machine> Compilable<'ast, M> for crate::ast::Static {
    type Output = Object;
    fn compile(&'ast self, machine: M, state: &mut GlobalState) -> Result<Self::Output, CompileError<'ast>> {
        todo!()
    }
}

impl<'a, 'ast, M: Machine> Compilable<'ast, M, FunctionScopeState<'a, 'ast>> for crate::ast::Block {
    type Output = Object;
    fn compile(&'ast self, machine: M, state: &mut FunctionScopeState<'a, 'ast>) -> Result<Self::Output, CompileError<'ast>> {
        self.statements
            .iter()
            .map(|item| item.compile(machine, state))
            .reduce(|a, b| Ok(a? + b?))
            .unwrap_or_else(|| Ok(Default::default()))
    }
}

impl<'a, 'ast, M: Machine> Compilable<'ast, M, FunctionScopeState<'a, 'ast>> for crate::ast::Statement {
    type Output = Object;
    fn compile(&'ast self, machine: M, state: &mut FunctionScopeState<'a, 'ast>) -> Result<Self::Output, CompileError<'ast>> {
        match self {
            crate::ast::Statement::LetStatement(stmt) => stmt.compile(machine, state),
            crate::ast::Statement::ExpressionStatement(expr) => Ok(expr.compile(machine, state)?.0),
            crate::ast::Statement::LoopStatement(stmt) => stmt.compile(machine, state),
            crate::ast::Statement::ReturnStatement(expr) => {
                todo!("compile return statement")
            },
        }
    }
}

impl<'a, 'ast, M: Machine> Compilable<'ast, M, FunctionScopeState<'a, 'ast>> for crate::ast::Let {
    type Output = Object;
    fn compile(&'ast self, machine: M, state: &mut FunctionScopeState<'a, 'ast>) -> Result<Self::Output, CompileError<'ast>> {
        let var = state.new_local(&self.variable.name, self.variable.mutable);
        let (mut code, initializer) = self.value.compile(machine, state)?;
        
        let mut regs = machine.usable_registers().into_iter();
        let reg = regs.next().expect("todo");

        code += machine.load_from(reg, &initializer).0;
        code += machine.store_to(reg, &var).0;

        Ok(code)
    }
}

impl<'a, 'ast, M: Machine> Compilable<'ast, M, FunctionScopeState<'a, 'ast>> for crate::ast::Loop {
    type Output = Object;
    fn compile(&'ast self, machine: M, state: &mut FunctionScopeState<'a, 'ast>) -> Result<Self::Output, CompileError<'ast>> {
        todo!()
    }
}

impl<'a, 'ast, M: Machine> Compilable<'ast, M, FunctionScopeState<'a, 'ast>> for crate::ast::Expression {
    // Returns the temporary that was created for the value of this expression.
    type Output = (Object, Rc<Variable>);
    fn compile(&'ast self, machine: M, state: &mut FunctionScopeState<'a, 'ast>) -> Result<Self::Output, CompileError<'ast>> {
        match self {
            crate::ast::Expression::AssignExpression { rhs, op, lhs } => {
                match (&*op.operator, op.checked) {
                    ("=", _) => {
                        let (mut code, lhs) = lhs.compile(machine, state)?;
                        let (rhs_code, rhs) = rhs.compile(machine, state)?;
                        code += rhs_code;

                        match (lhs.r#type, rhs.r#type) {
                            (Type::Integer, Type::Integer) => {},
                            _ => panic!("todo: typing"),
                        };

                        let mut regs = machine.usable_registers().into_iter();
                        let reg = regs.next().expect("todo");

                        code += machine.load_from(reg, &rhs).0;
                        code += machine.store_to(reg, &lhs).0;

                        Ok((code, rhs))
                    },
                    _ => todo!(),
                }
            },
            crate::ast::Expression::OrExpression { rhs, op, lhs } => todo!(),
            crate::ast::Expression::AndExpression { rhs, op, lhs } => todo!(),
            crate::ast::Expression::CompareExpression { rhs, op, lhs } => todo!(),
            crate::ast::Expression::AddExpression { rhs, op, lhs } => {
                let (mut code, lhs) = lhs.compile(machine, state)?;
                let (rhs_code, rhs) = rhs.compile(machine, state)?;
                code += rhs_code;

                match (lhs.r#type, rhs.r#type) {
                    (Type::Integer, Type::Integer) => {},
                    _ => panic!("todo: typing"),
                };

                let mut regs = machine.usable_registers().into_iter();
                let dst = regs.next().expect("todo");
                let src = regs.next().expect("todo");

                code += machine.load_from(dst, &lhs).0;
                code += machine.load_from(src, &rhs).0;
                match (&*op.operator, op.checked) {
                    ("+", false) => code += machine.add_assign(dst, src).0,
                    ("+", true)  => code += machine.checked_add_assign(dst, src).0,
                    ("-", false) => code += machine.sub_assign(dst, src).0,
                    ("-", true)  => code += machine.checked_sub_assign(dst, src).0,
                    _ => panic!("invalid operator"),
                };

                let result_var = state.new_temporary();

                code += machine.store_to(dst, &result_var).0;

                Ok((code, result_var))
            },
            crate::ast::Expression::MulExpression { rhs, op, lhs } => todo!(),
            crate::ast::Expression::ParenExpression { expr } => expr.compile(machine, state),
            crate::ast::Expression::CallExpression { function, args } => {
                let (mut code, func) = function.compile(machine, state)?;

                match func.r#type {
                    Type::Function => {},
                    _ => panic!("todo: typing"),
                };

                todo!()
            },
            crate::ast::Expression::VariableExpression { name } => {
                let var = state.get_variable(&name).ok_or(CompileError::UndefinedSymbol(&name))?;
                Ok((Default::default(), var))
            },
            crate::ast::Expression::BlockExpression { block } => todo!(),
            crate::ast::Expression::IntLiteralExpression { literal } => {
                let literal = literal.parse::<isize>().ok().ok_or(CompileError::Other("invalid integer literal".into()))?;
                let sym = Symbol::new_local();
                let (obj, var) = state.global_state.new_static(machine, literal, sym.clone());
                Ok((obj, var))
            },
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::compiler::CompileError;
    use super::Compilable;
    use crate::compiler::FunctionScopeState;
    use super::GlobalState;
    #[test]
    fn assignment_vars_1() {

        let source = r#"{
            x = y;
        }"#;
        let tokens = crate::ast::lex(source).unwrap().1;
        let ast = crate::ast::parsing::block(&tokens).unwrap().1;
        let mut global_state = GlobalState::default();
        let mut scope_state = FunctionScopeState::new(&mut global_state);
        assert_eq!(ast.compile(super::arch::Machine, &mut scope_state), Err(CompileError::UndefinedSymbol("x")));
    }
    #[test]
    fn assignment_vars_2() {

        let source = r#"{
            let x = 1;
            x = y;
        }"#;
        let tokens = crate::ast::lex(source).unwrap().1;
        let ast = crate::ast::parsing::block(&tokens).unwrap().1;
        let mut global_state = GlobalState::default();
        let mut scope_state = FunctionScopeState::new(&mut global_state);
        assert_eq!(ast.compile(super::arch::Machine, &mut scope_state), Err(CompileError::UndefinedSymbol("y")));
    }
    #[test]
    fn assignment_vars_3() {

        let source = r#"{
            let mut x = 1;
            let y = 2;
            x = y;
        }"#;
        let tokens = crate::ast::lex(source).unwrap().1;
        let ast = crate::ast::parsing::block(&tokens).unwrap().1;
        let mut global_state = GlobalState::default();
        let mut scope_state = FunctionScopeState::new(&mut global_state);
        let obj = ast.compile(super::arch::Machine, &mut scope_state).unwrap();
        let assembled = (obj.code + obj.data).assemble().unwrap();
        #[cfg(target_arch = "x86_64")]
        assert_eq!(assembled, [
            0x48, 0x8b, 0x05, 0x27, 0x00, 0x00, 0x00,       // mov .LCone(%rip),%rax
            0x48, 0x89, 0x84, 0x24, 0x00, 0x00, 0x00, 0x00, // mov %rax,x(%rsp)
            0x48, 0x8b, 0x05, 0x20, 0x00, 0x00, 0x00,       // mov .LCtwo(%rip),%rax
            0x48, 0x89, 0x84, 0x24, 0x08, 0x00, 0x00, 0x00, // mov %rax,y(%rsp)
            0x48, 0x8b, 0x84, 0x24, 0x08, 0x00, 0x00, 0x00, // mov y(%rsp),%rax
            0x48, 0x89, 0x84, 0x24, 0x00, 0x00, 0x00, 0x00, // mov %rax,x(%rsp)
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // .quad 1
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // .quad 2
        ]);
    }
    #[test]
    fn assignment_vars_4() {

        let source = r#"{
            let mut x = 1;
            let y = 2;
            x = x + y;
        }"#;
        let tokens = crate::ast::lex(source).unwrap().1;
        let ast = crate::ast::parsing::block(&tokens).unwrap().1;
        let mut global_state = GlobalState::default();
        let mut scope_state = FunctionScopeState::new(&mut global_state);
        let obj = ast.compile(super::arch::Machine, &mut scope_state).unwrap();
        let assembled = (obj.code + obj.data).assemble().unwrap();
        #[cfg(target_arch = "x86_64")]
        assert_eq!(assembled, [
            0x48, 0x8b, 0x05, 0x42, 0x00, 0x00, 0x00,       // mov .LCone(%rip),%rax
            0x48, 0x89, 0x84, 0x24, 0x00, 0x00, 0x00, 0x00, // mov %rax,x(%rsp)
            0x48, 0x8b, 0x05, 0x3b, 0x00, 0x00, 0x00,       // mov .LCtwo(%rip),%rax
            0x48, 0x89, 0x84, 0x24, 0x08, 0x00, 0x00, 0x00, // mov %rax,y(%rsp)
            0x48, 0x8b, 0x84, 0x24, 0x00, 0x00, 0x00, 0x00, // mov x(%rsp),%rax
            0x48, 0x8b, 0x8c, 0x24, 0x08, 0x00, 0x00, 0x00, // mov y(%rsp),%rcx
            0x48, 0x01, 0xc8,                               // add %rcx,%rax
            0x48, 0x89, 0x84, 0x24, 0x10, 0x00, 0x00, 0x00, // mov %rax,.Ltemp1
            0x48, 0x8b, 0x84, 0x24, 0x10, 0x00, 0x00, 0x00, // mov .Ltemp1,%rax
            0x48, 0x89, 0x84, 0x24, 0x00, 0x00, 0x00, 0x00, // mov %rax,x(%rsp)
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // .quad 1
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // .quad 2
        ]);
    }
}