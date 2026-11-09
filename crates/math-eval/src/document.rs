/// Evaluate a set of document expressions with shared variables and functions.
///
/// Dependencies are resolved by definition name regardless of position, so a
/// region can use a variable or function defined in any other region. Cycles
/// are reported as errors for every region that cannot be evaluated.
pub fn evaluate_document(items: &[(String, String)]) -> Vec<DocumentEvaluation> {
    evaluate_document_with_environment(items).0
}

/// Evaluate a document with user-defined constants available to every
/// expression. Custom constants are evaluated as document definitions before
/// the regular regions.
pub fn evaluate_document_with_constants(
    items: &[(String, String)],
    constants: &[(String, String)],
) -> Vec<DocumentEvaluation> {
    let valid = constants
        .iter()
        .filter(|(name, expression)| {
            !name.trim().is_empty() && !expression.trim().is_empty()
        })
        .collect::<Vec<_>>();
    if valid.is_empty() {
        return evaluate_document(items);
    }

    let mut combined = Vec::with_capacity(valid.len() + items.len());
    for (index, (name, expression)) in valid.iter().enumerate() {
        combined.push((
            format!("__constant_{index}"),
            format!("{} := {}", name.trim(), expression.trim()),
        ));
    }
    combined.extend(items.iter().cloned());

    let mut results = evaluate_document(&combined);
    results.split_off(valid.len())
}

pub fn evaluate_document_with_constants_and_preferences(
    items: &[(String, String)],
    constants: &[(String, String)],
    preferences: &UnitPreferences,
) -> Vec<DocumentEvaluation> {
    let mut results = evaluate_document_with_constants(items, constants);
    for evaluation in &mut results {
        if let Ok(value) = &mut evaluation.result {
            value.convert_units(preferences);
        }
    }
    results
}

pub fn evaluate_document_with_preferences(
    items: &[(String, String)],
    preferences: &UnitPreferences,
) -> Vec<DocumentEvaluation> {
    let mut results = evaluate_document(items);
    for evaluation in &mut results {
        if let Ok(value) = &mut evaluation.result {
            value.convert_units(preferences);
        }
    }
    results
}

pub fn evaluate_document_with_environment(
    items: &[(String, String)],
) -> (Vec<DocumentEvaluation>, Environment) {
    let count = items.len();
    let mut asts: Vec<Option<Expr>> = (0..count).map(|_| None).collect();
    let mut results: Vec<Option<Result<Value, EvalError>>> = (0..count).map(|_| None).collect();
    let mut variable_defs: HashMap<String, usize> = HashMap::new();
    let mut function_defs: HashMap<String, usize> = HashMap::new();
    let mut unit_defs: HashMap<String, usize> = HashMap::new();

    for (index, (_, text)) in items.iter().enumerate() {
        let text = text.trim();
        if text.is_empty() {
            results[index] = Some(Err(EvalError::Domain("empty expression".into())));
            continue;
        }

        match math_parse::parse(text) {
            Ok(expression) => {
                match &expression {
                    Expr::Assign { name, .. } => {
                        if variable_defs.contains_key(name) {
                            results[index] =
                                Some(Err(EvalError::DuplicateDefinition(name.clone())));
                            continue;
                        }
                        variable_defs.insert(name.clone(), index);
                    }
                    Expr::FunctionDef { name, .. } => {
                        if function_defs.contains_key(name) {
                            results[index] =
                                Some(Err(EvalError::DuplicateDefinition(name.clone())));
                            continue;
                        }
                        function_defs.insert(name.clone(), index);
                    }
                    Expr::UnitDef { name, .. } => {
                        if unit_defs.contains_key(name) {
                            results[index] =
                                Some(Err(EvalError::DuplicateDefinition(name.clone())));
                            continue;
                        }
                        unit_defs.insert(name.clone(), index);
                    }
                    _ => {}
                }
                asts[index] = Some(expression);
            }
            Err(error) => {
                results[index] = Some(Err(EvalError::Parse(error.to_string())));
            }
        }
    }

    let mut dependencies: Vec<Vec<usize>> = vec![Vec::new(); count];
    for (index, ast) in asts.iter().enumerate() {
        let Some(ast) = ast else {
            continue;
        };
        let (symbols, calls) = analyze_ast(ast);
        let mut edges = Vec::new();

        for name in symbols {
            if let Some(&definition) = variable_defs.get(&name) {
                edges.push(definition);
            } else if let Some(&definition) = unit_defs.get(&name) {
                edges.push(definition);
            }
        }
        for name in calls {
            if let Some(&definition) = function_defs.get(&name) {
                edges.push(definition);
            }
        }

        edges.sort_unstable();
        edges.dedup();
        dependencies[index] = edges;
    }

    let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); count];
    let mut indegree = vec![0usize; count];
    for (index, edges) in dependencies.iter().enumerate() {
        indegree[index] = edges.len();
        for &dependency in edges {
            dependents[dependency].push(index);
        }
    }

    let mut environment = Environment::default();
    let mut queue = VecDeque::new();
    for index in 0..count {
        if asts[index].is_some() && indegree[index] == 0 {
            queue.push_back(index);
        }
    }

    while let Some(index) = queue.pop_front() {
        let ast = asts[index].as_ref().expect("queued node has an AST");
        results[index] = Some(evaluate_with(ast, &mut environment));

        for &dependent in &dependents[index] {
            indegree[dependent] -= 1;
            if indegree[dependent] == 0 {
                queue.push_back(dependent);
            }
        }
    }

    for result in results.iter_mut() {
        if result.is_none() {
            *result = Some(Err(EvalError::DependencyCycle));
        }
    }

    let results = items
        .iter()
        .enumerate()
        .map(|(index, (id, _))| DocumentEvaluation {
            id: id.clone(),
            result: results[index]
                .take()
                .unwrap_or(Err(EvalError::DependencyCycle)),
        })
        .collect::<Vec<_>>();

    (results, environment)
}

fn analyze_ast(expression: &Expr) -> (HashSet<String>, HashSet<String>) {
    let mut symbols = HashSet::new();
    let mut calls = HashSet::new();
    collect_dependencies(expression, &HashSet::new(), &mut symbols, &mut calls);
    (symbols, calls)
}

fn collect_dependencies(
    expression: &Expr,
    locals: &HashSet<String>,
    symbols: &mut HashSet<String>,
    calls: &mut HashSet<String>,
) {
    match expression {
        Expr::Number(_) => {}
        Expr::Symbol(name) => {
            if !locals.contains(name) {
                symbols.insert(name.clone());
            }
        }
        Expr::Unary { expr, .. } => collect_dependencies(expr, locals, symbols, calls),
        Expr::Binary { lhs, rhs, .. } => {
            collect_dependencies(lhs, locals, symbols, calls);
            collect_dependencies(rhs, locals, symbols, calls);
        }
        Expr::Call { name, args } => {
            calls.insert(name.clone());
            let lower = name.to_ascii_lowercase();
            if matches!(
                lower.as_str(),
                "for" | "para" | "sum" | "sumatoria" | "product" | "producto"
            ) && (args.len() == 4 || args.len() == 3)
            {
                if let Expr::Symbol(variable) = &args[0] {
                    if args.len() == 4 {
                        collect_dependencies(&args[1], locals, symbols, calls);
                        collect_dependencies(&args[2], locals, symbols, calls);
                        let mut nested = locals.clone();
                        nested.insert(variable.clone());
                        collect_dependencies(&args[3], &nested, symbols, calls);
                    } else {
                        collect_dependencies(&args[1], locals, symbols, calls);
                        let mut nested = locals.clone();
                        nested.insert(variable.clone());
                        collect_dependencies(&args[2], &nested, symbols, calls);
                    }
                    return;
                }
            }
            for arg in args {
                collect_dependencies(arg, locals, symbols, calls);
            }
        }
        Expr::List(items) => {
            for item in items {
                collect_dependencies(item, locals, symbols, calls);
            }
        }
        Expr::Matrix(rows) => {
            for row in rows {
                for item in row {
                    collect_dependencies(item, locals, symbols, calls);
                }
            }
        }
        Expr::Index { target, indices } => {
            collect_dependencies(target, locals, symbols, calls);
            for index in indices {
                collect_dependencies(index, locals, symbols, calls);
            }
        }
        Expr::Assign { value, .. } => {
            collect_dependencies(value, locals, symbols, calls);
        }
        Expr::FunctionDef { params, body, .. } => {
            let mut nested = locals.clone();
            nested.extend(params.iter().cloned());
            collect_dependencies(body, &nested, symbols, calls);
        }
        Expr::UnitDef { value, .. } => {
            collect_dependencies(value, locals, symbols, calls);
        }
    }
}
