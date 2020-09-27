//! The main analyzer structure.

use crate::*;
use rslint_parser::{ast::*, SyntaxKind::*, SyntaxNode, SyntaxNodeExt};
use std::fmt::Debug;
use std::rc::{Rc, Weak};
use tracing::{instrument, trace};

// We opted to use unsafe Rc mutation instead of using an `Rc<RefCell<T>>` for the sake
// of interface, every single use of unsafe has a comment explaining why it is safe.
// scope is immutable except when it is being created, therefore it is perfectly safe for
// the analyzer to assume nothing is going to dereference anything in the scope while it is modifying it.
// This is further ensured by the fact that the analyzer is completely private.
// Furthermore, the analyzer operates sequentially, therefore it is nearly impossible for a read to occur while
// we are mutating something in the scope.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PatternContainer {
    Const,
    Let,
    Var,
    Param,
}

#[derive(Debug, Clone)]
struct AnalyzerState {
    /// References to variables which may or may not be undefined (for var and functions)
    maybe_hoisted_refs: Vec<Rc<VariableRef>>,
}

#[derive(Debug)]
pub(crate) struct Analyzer {
    pub scope: Rc<Scope>,
    state: AnalyzerState,
}

impl Analyzer {
    pub(crate) fn new_root(root: SyntaxNode) -> Self {
        Self {
            scope: Rc::new(Scope {
                node: root,
                kind: ScopeKind::Global,
                variables: vec![],
                var_refs: vec![],
                children: vec![],
                parent: None,
                unreachable: false,
            }),
            state: AnalyzerState {
                maybe_hoisted_refs: vec![],
            },
        }
    }

    fn base_scope(node: SyntaxNode, kind: ScopeKind) -> Scope {
        Scope {
            node,
            kind,
            variables: vec![],
            var_refs: vec![],
            children: vec![],
            parent: None,
            unreachable: false,
        }
    }

    #[instrument]
    pub(crate) fn new_subscope(node: SyntaxNode, parent: Rc<Scope>, kind: ScopeKind) -> Self {
        let mut base = Self::base_scope(node, kind);
        base.parent = Some(Rc::downgrade(&parent));
        base.variables = parent.variables.clone();
        base.variables
            .iter_mut()
            // Safety: no other immutable reads of the Rc occur while we are mutating var.
            .for_each(|var| unsafe {
                (*(Rc::as_ptr(var) as *mut VariableBinding)).inherited = true
            });

        Self {
            scope: Rc::new(base),
            state: AnalyzerState {
                maybe_hoisted_refs: vec![],
            },
        }
    }

    /// Consume the analyzer and turn it into an analyzed scope.
    ///
    /// # Panics
    /// Panics if the scope is not global scope, because then there will be strong
    /// references to it (from parent scope children) and it is not safe/possible
    /// to unwrap its Rc
    pub(crate) fn end(self) -> Scope {
        assert_eq!(self.scope.kind, ScopeKind::Global);
        Rc::try_unwrap(self.scope).unwrap()
    }

    #[instrument(skip(start_node))]
    pub(crate) fn analyze(&mut self, start_node: impl Into<Option<SyntaxNode>>) {
        let root = start_node.into().unwrap_or_else(|| self.scope.node.clone());

        root.descendants_with(&mut |child| {
            match child.kind() {
                IMPORT_DECL => self.bind_import_declaration(&child),
                CATCH_CLAUSE => {
                    let mut new =
                        Self::new_subscope(child.clone(), self.scope.clone(), ScopeKind::Catch);

                    let new_cloned = new.scope.clone();

                    // Safety: no reads occur while we are adding the child.
                    unsafe {
                        (*(Rc::as_ptr(&self.scope) as *mut Scope))
                            .children
                            .push(new_cloned);
                    }
                    new.maybe_bind_catch_ident(&child.to());
                    new.analyze(None);
                }
                BLOCK_STMT => {
                    let mut new =
                        Self::new_subscope(child.clone(), self.scope.clone(), ScopeKind::Block);

                    let new_cloned = new.scope.clone();

                    // Safety: no reads occur while we are adding the child.
                    unsafe {
                        (*(Rc::as_ptr(&self.scope) as *mut Scope))
                            .children
                            .push(new_cloned);
                    }
                    new.analyze(None);
                }
                VAR_DECL => {
                    let start = self.scope.variables.len();
                    let decl = child.to::<VarDecl>();
                    for declarator in decl.declared() {
                        if let Some(pat) = declarator.pattern() {
                            let container = if decl.is_const() {
                                PatternContainer::Const
                            } else if decl.is_let() {
                                PatternContainer::Let
                            } else {
                                PatternContainer::Var
                            };

                            self.bind_pat(pat, container, false, false);
                        }
                        if let Some(ref val) = declarator.value() {
                            self.analyze_expr(val);
                        }
                    }
                    self.record_hoisted_declarations(start);
                }
                FN_DECL => self.enter_function_scope(child.clone()),
                _ if child.is::<Expr>() => self.analyze_expr(&child.to()),
                _ => return true,
            }
            false
        });
    }

    pub(crate) fn analyze_expr(&mut self, expr: &Expr) {
        let res = self.analyze_expr_inner(expr.syntax().clone());
        if res {
            expr.syntax()
                .descendants_with(&mut |child| self.analyze_expr_inner(child.clone()));
        }
    }

    fn analyze_expr_inner(&mut self, child: SyntaxNode) -> bool {
        match child.kind() {
            ARROW_EXPR => {
                self.enter_arrow_function_scope(child.clone());
                false
            }
            FN_EXPR => {
                self.enter_function_scope(child.clone());
                false
            }
            DOT_EXPR => {
                if let Some(lhs) = child.to::<DotExpr>().object() {
                    self.analyze_expr(&lhs);
                }
                false
            }
            BRACKET_EXPR => {
                if let Some(name) = child
                    .to::<BracketExpr>()
                    .object()
                    .and_then(|expr| expr.syntax().try_to::<Name>())
                {
                    self.record_var_use(&name, false);
                }
                true
            }
            _ if child.is::<Pattern>()
                && child.parent().map_or(false, |node| node.is::<AssignExpr>()) =>
            {
                self.bind_pat(child.to(), PatternContainer::Const, true, true);
                if let Some(rhs) = child.parent().unwrap().to::<AssignExpr>().rhs() {
                    self.analyze_expr(&rhs);
                }
                false
            }
            NAME => {
                self.record_var_use(&child.to(), false);
                true
            }
            _ if child.is::<ObjectExpr>() => {
                let obj = child.to::<ObjectExpr>();
                for elem in obj.props() {
                    match elem {
                        ObjectProp::IdentProp(prop) => {
                            if let Some(ref name) = prop.name() {
                                self.record_var_use(name, false);
                            }
                        }
                        ObjectProp::InitializedProp(prop) => {
                            if let Some(ref name) = prop.key() {
                                self.record_var_use(name, false);
                            }
                            if let Some(ref rhs) = prop.value() {
                                self.analyze_expr(rhs);
                            }
                        }
                        ObjectProp::Getter(prop) => {
                            let mut new = Self::new_subscope(
                                prop.syntax().clone(),
                                self.scope.clone(),
                                ScopeKind::Getter,
                            );
                            let new_cloned = new.scope.clone();

                            // Safety: no reads occur while we are adding the child.
                            unsafe {
                                (*(Rc::as_ptr(&self.scope) as *mut Scope))
                                    .children
                                    .push(new_cloned);
                            }

                            if let Some(PropName::Ident(ref name)) = prop.key() {
                                new.bind_var(BindingKind::Function, name, new.shadow(name));
                            }
                            if let Some(body) = prop.body() {
                                new.analyze(body.syntax().clone());
                            }
                        }
                        ObjectProp::Setter(prop) => {
                            let mut new = Self::new_subscope(
                                prop.syntax().clone(),
                                self.scope.clone(),
                                ScopeKind::Setter,
                            );
                            let new_cloned = new.scope.clone();

                            // Safety: no reads occur while we are adding the child.
                            unsafe {
                                (*(Rc::as_ptr(&self.scope) as *mut Scope))
                                    .children
                                    .push(new_cloned);
                            }

                            if let Some(PropName::Ident(ref name)) = prop.key() {
                                new.bind_var(BindingKind::Function, name, new.shadow(name));
                            }
                            if let Some(params) = prop.parameters() {
                                new.bind_params(params);
                            }
                            if let Some(body) = prop.body() {
                                new.analyze(body.syntax().clone());
                            }
                        }
                        ObjectProp::Method(method) => {
                            let mut new = Self::new_subscope(
                                method.syntax().clone(),
                                self.scope.clone(),
                                ScopeKind::Function,
                            );
                            let new_cloned = new.scope.clone();

                            // Safety: no reads occur while we are adding the child.
                            unsafe {
                                (*(Rc::as_ptr(&self.scope) as *mut Scope))
                                    .children
                                    .push(new_cloned);
                            }

                            if let Some(PropName::Ident(ref name)) = method.name() {
                                new.bind_var(BindingKind::Function, name, new.shadow(name));
                            }
                            if let Some(params) = method.parameters() {
                                new.bind_params(params);
                            }
                            if let Some(body) = method.body() {
                                new.analyze(body.syntax().clone());
                            }
                        }
                        _ => {}
                    }
                }
                false
            }
            _ => true,
        }
    }

    fn record_hoisted_declarations(&mut self, declared_vars_start: usize) {
        let vars = &self.scope.variables;
        let mut declared = vars.iter().skip(declared_vars_start);
        for maybe_valid in &self.state.maybe_hoisted_refs {
            let corresponding_decl = declared
                .find(|x| x.name.as_str() == maybe_valid.node.trimmed_text())
                .cloned();

            if corresponding_decl.clone().map_or(
                false,
                |x| matches!(x.kind, BindingKind::Const(_) | BindingKind::Let(_)),
            ) {
                // Safety: no other immutable reads occur while we change the hoisted declaration
                unsafe {
                    (*(Rc::as_ptr(maybe_valid) as *mut VariableRef)).hoisted_declaration =
                        corresponding_decl.map(|this| Rc::downgrade(&this));
                }
            } else if let Some(decl) = corresponding_decl {
                for var_ref in decl
                    .references
                    .iter()
                    .map(|weak| weak.upgrade().expect("Weak dropped in scope analysis"))
                {
                    // Safety: same as before, no immutable reads occur while the pointer exists
                    unsafe {
                        (*(Rc::as_ptr(&var_ref) as *mut VariableRef)).declaration =
                            Some(Rc::downgrade(&decl));
                    }
                }
            }
        }
    }

    fn record_var_use(&mut self, name: &Name, assign: bool) {
        let usage = if name
            .syntax()
            .parent()
            .map_or(false, |x| x.kind() == NEW_EXPR)
        {
            VariableUsageKind::Construct
        } else if name
            .syntax()
            .parent()
            .map_or(false, |x| x.kind() == CALL_EXPR)
        {
            VariableUsageKind::Call
        } else if assign {
            VariableUsageKind::Write
        } else {
            VariableUsageKind::Read
        };

        let var_ref = Rc::new(VariableRef {
            node: name.syntax().clone(),
            usage,
            declaration: self
                .scope
                .variables
                .iter()
                .find(|var| &var.name == name.ident_token().unwrap().text())
                .map(|this| Rc::downgrade(this)),
            hoisted_declaration: None,
        });

        if var_ref.declaration.is_none() {
            self.state.maybe_hoisted_refs.push(var_ref.clone());
        } else {
            let var_ref_rc = var_ref
                .declaration
                .clone()
                .unwrap()
                .upgrade()
                .expect("Weak ref to declaration is dropped prematurely");

            unsafe {
                let weak = Rc::downgrade(&var_ref);
                // Safety: No immutable reads occur while we are pushing to the references, since we downgrade the rc
                // before we create the pointer to the underlying Rc
                (*(Rc::as_ptr(&var_ref_rc) as *mut VariableBinding))
                    .references
                    .push(weak);
            }
        }

        // Safety: no reads occur while we are pushing to the scope.
        unsafe {
            (*(Rc::as_ptr(&self.scope) as *mut Scope))
                .var_refs
                .push(var_ref);
        }
    }

    #[instrument]
    pub(crate) fn enter_function_scope(&mut self, node: SyntaxNode) {
        let mut new = Self::new_subscope(node.clone(), self.scope.clone(), ScopeKind::Function);
        let new_cloned = new.scope.clone();

        // Safety: no reads occur while we are adding the child.
        unsafe {
            (*(Rc::as_ptr(&self.scope) as *mut Scope))
                .children
                .push(new_cloned);
        }

        if let Some(ref name) = node.to::<FnDecl>().name() {
            new.bind_var(BindingKind::Function, name, new.shadow(name));
        }
        if let Some(params) = node.to::<FnDecl>().parameters() {
            new.bind_params(params);
        }
        new.analyze(node.to::<FnDecl>().body().map(|x| x.syntax().clone()))
    }

    #[instrument]
    pub(crate) fn enter_arrow_function_scope(&mut self, node: SyntaxNode) {
        let mut new = Self::new_subscope(node.clone(), self.scope.clone(), ScopeKind::Arrow);
        let new_cloned = new.scope.clone();

        // Safety: no reads occur while we are adding the child.
        unsafe {
            (*(Rc::as_ptr(&self.scope) as *mut Scope))
                .children
                .push(new_cloned);
        }

        if let Some(params) = node.to::<ArrowExpr>().params() {
            match params {
                ArrowExprParams::Name(name) => {
                    new.bind_var(
                        BindingKind::Param(PatternBindingKind::Literal),
                        &name,
                        new.shadow(&name),
                    );
                }
                ArrowExprParams::ParameterList(list) => new.bind_params(list),
            }
        }
        match node.to::<ArrowExpr>().body() {
            Some(ExprOrBlock::Block(block)) => new.analyze(block.syntax().clone()),
            Some(ExprOrBlock::Expr(expr)) => new.analyze(expr.syntax().clone()),
            _ => {}
        }
    }

    fn bind_params(&mut self, params: ParameterList) {
        for param in params.parameters() {
            self.bind_pat(param, PatternContainer::Param, false, false);
        }
    }

    #[instrument]
    pub(crate) fn maybe_bind_catch_ident(&mut self, clause: &CatchClause) {
        if let Some(ref name) = clause.error() {
            self.bind_var(BindingKind::CatchClause, name, self.shadow(name));
        }
    }

    #[instrument]
    pub(crate) fn bind_import_declaration(&mut self, node: &SyntaxNode) {
        let import_stmt = node.to::<ImportDecl>();
        let source = import_stmt
            .source()
            .and_then(|x| x.inner_string_text())
            .map(|x| SmolStr::from(x.to_string()));

        for import in import_stmt.imports() {
            match import {
                ImportClause::Name(name) => self.bind_var(
                    BindingKind::Import(ImportBindingKind::LiteralImport, source.clone()),
                    &name,
                    self.shadow(&name),
                ),
                ImportClause::NamedImports(named) => {
                    for specifier in named.specifiers() {
                        if let Some(name) = specifier.name() {
                            self.bind_var(
                                BindingKind::Import(
                                    ImportBindingKind::DestructuredImport(
                                        name.to::<Name>().ident_token().map(|x| x.text().clone()),
                                    ),
                                    source.clone(),
                                ),
                                &name.to(),
                                self.shadow(&name.to()),
                            );
                        }
                    }
                }
                ImportClause::WildcardImport(wildcard) => {
                    if let Some(ref name) = wildcard.alias() {
                        self.bind_var(
                            BindingKind::Import(ImportBindingKind::NamedWildcard, source.clone()),
                            name,
                            self.shadow(&name),
                        )
                    }
                }
            }
        }
    }

    #[instrument]
    fn bind_pat(
        &mut self,
        pat: Pattern,
        container: PatternContainer,
        record_usage: bool,
        assign: bool,
    ) {
        match pat {
            Pattern::SinglePattern(single_pat) => {
                if !record_usage {
                    self.bind_var(
                        Self::pattern_container_to_binding_kind(
                            container,
                            PatternBindingKind::Literal,
                        ),
                        &single_pat.name().unwrap(),
                        self.shadow(&single_pat.name().unwrap()),
                    );
                } else {
                    self.record_var_use(&single_pat.name().unwrap(), assign);
                }
            }
            Pattern::ArrayPattern(array_pat) => {
                for elem in array_pat.elements() {
                    if let Pattern::SinglePattern(single_pat) = elem {
                        if !record_usage {
                            self.bind_var(
                                Self::pattern_container_to_binding_kind(
                                    container,
                                    PatternBindingKind::Array,
                                ),
                                &single_pat.name().unwrap(),
                                self.shadow(&single_pat.name().unwrap()),
                            );
                        } else {
                            self.record_var_use(&single_pat.name().unwrap(), assign);
                        }
                    } else {
                        self.bind_pat(elem, container, record_usage, assign);
                    }
                }
            }
            Pattern::ObjectPattern(object_pat) => {
                for elem in object_pat.elements() {
                    match elem {
                        ObjectPatternProp::SinglePattern(single_pat) => {
                            if !record_usage {
                                self.bind_var(
                                    Self::pattern_container_to_binding_kind(
                                        container,
                                        PatternBindingKind::Object,
                                    ),
                                    &single_pat.name().unwrap(),
                                    self.shadow(&single_pat.name().unwrap()),
                                );
                            } else {
                                self.record_var_use(&single_pat.name().unwrap(), assign);
                            };
                        }
                        ObjectPatternProp::KeyValuePattern(key_val) => {
                            if let Some(val) = key_val.value() {
                                self.bind_pat(val, container, record_usage, assign);
                            }
                        }
                        ObjectPatternProp::AssignPattern(assign_pat) => {
                            if let Some(key) = assign_pat.key() {
                                self.bind_pat(key, container, record_usage, assign);
                            }
                        }
                        ObjectPatternProp::RestPattern(rest) => {
                            if let Some(val) = rest.pat() {
                                self.bind_pat(val, container, record_usage, assign);
                            }
                        }
                    }
                }
            }
            Pattern::RestPattern(rest_pat) => {
                if let Some(val) = rest_pat.pat() {
                    self.bind_pat(val, container, record_usage, assign);
                }
            }
            Pattern::AssignPattern(assign_pat) => {
                if let Some(key) = assign_pat.key() {
                    self.bind_pat(key, container, record_usage, assign);
                }
            }
        };
    }

    fn pattern_container_to_binding_kind(
        container: PatternContainer,
        pat_kind: PatternBindingKind,
    ) -> BindingKind {
        match container {
            PatternContainer::Const => BindingKind::Const(pat_kind),
            PatternContainer::Let => BindingKind::Let(pat_kind),
            PatternContainer::Var => BindingKind::Var(pat_kind),
            PatternContainer::Param => BindingKind::Param(pat_kind),
        }
    }

    #[instrument]
    pub(crate) fn shadow(&self, name: &Name) -> Vec<Weak<VariableBinding>> {
        self.scope
            .variables
            .iter()
            .find(|var| var.name == name.to_string())
            .map(|this| {
                let mut shadow = this.clone().shadows.clone();
                shadow.push(Rc::downgrade(this));
                shadow
            })
            .unwrap_or_default()
    }

    /// Bind an identifier as a variable.
    #[instrument(skip(shadow))]
    pub(crate) fn bind_var(
        &mut self,
        kind: BindingKind,
        name: &Name,
        shadow: impl Into<Vec<Weak<VariableBinding>>>,
    ) {
        let var = VariableBinding {
            node: Some(name.syntax().clone()),
            name: name.ident_token().unwrap().text().clone(),
            references: vec![],
            shadows: shadow.into(),
            kind: kind.clone(),
            inherited: false,
        };

        trace!(
            "binding variable {} ({:?}) in a {:?} scope",
            name,
            kind,
            self.scope.kind
        );

        unsafe {
            (*(Rc::as_ptr(&self.scope) as *mut Scope))
                .variables
                .push(Rc::new(var))
        }
    }
}
