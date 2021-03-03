use crate::{ir::*, util::*};
use ast::Expr;
use rslint_parser::{
    ast::{
        CatchClause, ClassDecl, FnDecl, Getter, Method, ParameterList, Pattern, PropName, Setter,
        VarDecl,
    },
    *,
};
use std::{
    cell::RefCell,
    ops::DerefMut,
    rc::{Rc, Weak},
    sync::atomic::{AtomicUsize, Ordering},
};
use SyntaxKind::*;

static VAR_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

const BLOCKLIKE_SCOPES: &[ScopeKind] = &[
    ScopeKind::Block,
    ScopeKind::Catch,
    ScopeKind::Loop,
    ScopeKind::With,
];

type Checkpoint = (Rc<RefCell<Scope>>, Rc<RefCell<Scope>>);

#[derive(Clone)]
pub(crate) struct Analyzer {
    pub(crate) cur_scope: Rc<RefCell<Scope>>,
    var_scope: Rc<RefCell<Scope>>,
}

impl Analyzer {
    pub(crate) fn from_root(root: SyntaxNode) -> Self {
        let scope = Rc::new(RefCell::new(Scope {
            node: root,
            kind: ScopeKind::Global,
            var_refs: vec![],
            variables: vec![],
            parent: None,
            strict: false,
            children: vec![],
        }));
        Self {
            cur_scope: scope.clone(),
            var_scope: scope,
        }
    }

    fn checkpoint(&self) -> Checkpoint {
        (self.cur_scope.clone(), self.var_scope.clone())
    }

    fn load(&mut self, checkpoint: Checkpoint) {
        self.cur_scope = checkpoint.0;
        self.var_scope = checkpoint.1;
    }

    fn enter_new_scope(&mut self, node: SyntaxNode, kind: ScopeKind) -> Checkpoint {
        let mut scope = Scope {
            node,
            kind,
            var_refs: vec![],
            variables: vec![],
            parent: Some(Rc::downgrade(&self.cur_scope)),
            strict: false,
            children: vec![],
        };

        scope
            .variables
            .extend(self.cur_scope.borrow().variables.clone());

        let rc_scope = Rc::new(RefCell::new(scope));
        self.cur_scope.borrow_mut().children.push(rc_scope.clone());
        let checkpoint = self.checkpoint();

        if !BLOCKLIKE_SCOPES.contains(&kind) {
            self.var_scope = rc_scope.clone();
        }

        self.cur_scope = rc_scope;
        checkpoint
    }

    pub(crate) fn analyze_cur_scope(&mut self) {
        let node = self.cur_scope.borrow().node.clone();
        self.analyze(node);
    }

    fn analyze_node(&mut self, node: &SyntaxNode) -> bool {
        match node.kind() {
            BLOCK_STMT => {
                let checkpoint = self.enter_new_scope(node.clone(), ScopeKind::Block);
                self.analyze(node.clone());
                self.load(checkpoint);
                false
            }
            VAR_DECL => {
                let decl = node.to::<VarDecl>();
                for declarator in decl.declared() {
                    if let Some(pat) = declarator.pattern() {
                        let pat_kind = match pat {
                            Pattern::SinglePattern(_) | Pattern::AssignPattern(_) => {
                                PatternBindingKind::Literal
                            }
                            Pattern::ArrayPattern(_) => PatternBindingKind::Array,
                            Pattern::ObjectPattern(_) => PatternBindingKind::Object,
                            _ => unreachable!(),
                        };

                        let kind = if decl.is_const() {
                            BindingKind::Const(pat_kind)
                        } else if decl.is_let() {
                            BindingKind::Let(pat_kind)
                        } else {
                            BindingKind::Var(pat_kind)
                        };

                        // doing operations on the clone works because everything uses interior mutability
                        let mut clone = self.clone();
                        expand_pattern(
                            pat,
                            &mut |name| {
                                self.bind_var(
                                    if decl.is_var() {
                                        self.var_scope.clone()
                                    } else {
                                        self.cur_scope.clone()
                                    },
                                    node.clone(),
                                    kind.clone(),
                                    declarator.value(),
                                    name.to_string(),
                                );
                            },
                            &mut |expr| clone.analyze(expr.syntax().clone()),
                        );

                        if let Some(val) = declarator.value() {
                            self.analyze(val.syntax().clone());
                        }
                    }
                }
                false
            }
            FN_DECL => {
                let decl = node.to::<FnDecl>();
                if let Some(name) = decl.name() {
                    self.bind_var(
                        self.cur_scope.clone(),
                        decl.syntax().clone(),
                        BindingKind::Function,
                        None,
                        name.to_string(),
                    );
                }
                let checkpoint = self.enter_new_scope(decl.syntax().clone(), ScopeKind::Function);

                self.bind_var(
                    self.cur_scope.clone(),
                    decl.syntax().clone(),
                    BindingKind::Arguments,
                    None,
                    "arguments".into(),
                );
                if let Some(list) = decl.parameters() {
                    self.bind_parameter_list(node.clone(), list);
                }
                if let Some(body) = decl.body() {
                    self.analyze(body.syntax().clone());
                }
                self.load(checkpoint);
                false
            }
            CATCH_CLAUSE => {
                let checkpoint = self.enter_new_scope(node.clone(), ScopeKind::Catch);
                let clause = node.to::<CatchClause>();
                if let Some(pat) = clause.error() {
                    let mut clone = self.clone();
                    expand_pattern(
                        pat,
                        &mut |name| {
                            self.bind_var(
                                self.cur_scope.clone(),
                                clause.syntax().clone(),
                                BindingKind::CatchClause,
                                None,
                                name.to_string(),
                            );
                        },
                        &mut |expr| clone.analyze(expr.syntax().clone()),
                    );
                }
                if let Some(body) = clause.cons() {
                    self.analyze(body.syntax().clone());
                }
                self.load(checkpoint);
                false
            }
            CLASS_DECL => {
                let checkpoint = self.enter_new_scope(node.clone(), ScopeKind::Class);
                let decl = node.to::<ClassDecl>();

                if let Some(name) = decl.name() {
                    self.bind_var(
                        self.cur_scope.clone(),
                        node.clone(),
                        BindingKind::Class,
                        None,
                        name.to_string(),
                    );
                }

                if let Some(parent) = decl.parent() {
                    self.analyze_node(parent.syntax());
                }

                if let Some(body) = decl.body() {
                    self.analyze(body.syntax().clone());
                }
                self.load(checkpoint);
                false
            }
            METHOD => {
                let checkpoint = self.enter_new_scope(node.clone(), ScopeKind::Method);
                let method = node.to::<Method>();

                if let Some(PropName::Ident(name)) = method.name() {
                    self.bind_var(
                        self.cur_scope.clone(),
                        node.clone(),
                        BindingKind::Method,
                        None,
                        name.to_string(),
                    );
                }

                if let Some(list) = method.parameters() {
                    self.bind_parameter_list(node.clone(), list);
                }

                if let Some(body) = method.body() {
                    self.analyze(body.syntax().clone());
                }
                self.load(checkpoint);
                false
            }
            GETTER => {
                let checkpoint = self.enter_new_scope(node.clone(), ScopeKind::Getter);
                let getter = node.to::<Getter>();

                if let Some(PropName::Ident(name)) = getter.key() {
                    self.bind_var(
                        self.cur_scope.clone(),
                        node.clone(),
                        BindingKind::Getter,
                        None,
                        name.to_string(),
                    );
                }

                if let Some(list) = getter.parameters() {
                    self.bind_parameter_list(node.clone(), list);
                }

                if let Some(body) = getter.body() {
                    self.analyze(body.syntax().clone());
                }
                self.load(checkpoint);
                false
            }
            SETTER => {
                let checkpoint = self.enter_new_scope(node.clone(), ScopeKind::Setter);
                let setter = node.to::<Setter>();

                if let Some(PropName::Ident(name)) = setter.key() {
                    self.bind_var(
                        self.cur_scope.clone(),
                        node.clone(),
                        BindingKind::Setter,
                        None,
                        name.to_string(),
                    );
                }

                if let Some(list) = setter.parameters() {
                    self.bind_parameter_list(node.clone(), list);
                }

                if let Some(body) = setter.body() {
                    self.analyze(body.syntax().clone());
                }
                self.load(checkpoint);
                false
            }
            _ => true,
        }
    }

    pub fn analyze(&mut self, node: SyntaxNode) {
        node.descendants_with(&mut |n| self.analyze_node(n));
    }

    fn bind_parameter_list(&mut self, node: SyntaxNode, list: ParameterList) {
        let mut clone = self.clone();
        for pat in list.parameters() {
            let pat_kind = match pat {
                Pattern::SinglePattern(_) | Pattern::AssignPattern(_) => {
                    PatternBindingKind::Literal
                }
                Pattern::ArrayPattern(_) => PatternBindingKind::Array,
                Pattern::ObjectPattern(_) => PatternBindingKind::Object,
                _ => unreachable!(),
            };

            expand_pattern(
                pat,
                &mut |name| {
                    self.bind_var(
                        self.cur_scope.clone(),
                        node.clone(),
                        BindingKind::Param(pat_kind.clone()),
                        None,
                        name.to_string(),
                    );
                },
                &mut |expr| clone.analyze(expr.syntax().clone()),
            );
        }
    }

    fn bind_var(
        &mut self,
        scope: Rc<RefCell<Scope>>,
        node: SyntaxNode,
        kind: BindingKind,
        initial_value: Option<Expr>,
        name: String,
    ) -> Rc<RefCell<VariableBinding>> {
        let scope_ref = scope.borrow_mut();
        // if the scope already contains the variable then we are sure that it has already been propagated
        // to children, therefore we don't have to propagate it

        // can't inline this into the if let Some because we need to drop scope_ref after the clause to avoid refcell panics
        let binding = scope_ref
            .variables
            .iter()
            .find(|x| x.borrow().name == name)
            .cloned();
        if let Some(binding) = binding {
            drop(scope_ref);
            let scope_ref = scope.borrow();
            let mut binding_ref = binding.borrow_mut();
            // shadowing
            let upgrade = Weak::upgrade(&binding_ref.scope).unwrap();
            let target_scope = upgrade.borrow();
            if scope_ref.is_child_of(&*target_scope) {
                drop((binding_ref, scope_ref, target_scope));
                let binding = Rc::new(RefCell::new(VariableBinding {
                    declarations: vec![VariableDeclaration {
                        node,
                        initial_value,
                        kind,
                    }],
                    name,
                    references: vec![],
                    id: VAR_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
                    scope: Rc::downgrade(&scope),
                }));
                self.push_binding_to_scope(scope.borrow_mut(), binding, true);
            } else if !binding_ref
                .declarations
                .iter()
                .any(|decl| decl.node == node)
            {
                binding_ref.declarations.push(VariableDeclaration {
                    node,
                    initial_value,
                    kind,
                });
            }

            binding.clone()
        } else {
            let binding = Rc::new(RefCell::new(VariableBinding {
                declarations: vec![VariableDeclaration {
                    node,
                    initial_value,
                    kind,
                }],
                name,
                references: vec![],
                id: VAR_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
                scope: Rc::downgrade(&scope),
            }));
            self.push_binding_to_scope(scope_ref, binding.clone(), false);
            binding
        }
    }

    fn push_binding_to_scope(
        &mut self,
        mut scope: impl DerefMut<Target = Scope>,
        binding: Rc<RefCell<VariableBinding>>,
        clear_duplicate: bool,
    ) {
        let scope = scope.deref_mut();
        if clear_duplicate {
            if let Some(existing) = scope
                .variables
                .iter_mut()
                .find(|bind| bind.borrow().name == binding.borrow().name)
            {
                *existing = binding.clone();
            } else {
                scope.variables.push(binding.clone());
            }
        } else {
            scope.variables.push(binding.clone());
        }
        for child in scope.children.iter() {
            self.push_binding_to_scope((*child).borrow_mut(), binding.clone(), clear_duplicate);
        }
    }
}
