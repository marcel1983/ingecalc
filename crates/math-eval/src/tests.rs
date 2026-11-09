use super::*;
use math_parse::parse;

fn evaluate_text(text: &str) -> Result<Value, EvalError> {
    let expression = parse(text).map_err(|error| EvalError::Domain(error.to_string()))?;
    evaluate(&expression)
}

fn value_of(items: &[(String, String)], id: &str) -> Option<String> {
    evaluate_document(items)
        .into_iter()
        .find(|evaluation| evaluation.id == id)
        .and_then(|evaluation| evaluation.result.ok())
        .map(|value| value.display())
}

#[test]
fn evaluates_arithmetic_and_rationals() {
    assert_eq!(evaluate_text("2+2").unwrap().display(), "4");
    assert_eq!(evaluate_text("1/3 + 1/6").unwrap().display(), "1/2");
    assert_eq!(evaluate_text("2/4").unwrap().display(), "1/2");
    assert_eq!(evaluate_text("(1/3)*3").unwrap().display(), "1");
    assert_eq!(evaluate_text("2^10").unwrap().display(), "1024");
}

#[test]
fn evaluates_complex_numbers() {
    assert_eq!(evaluate_text("i^2").unwrap().display(), "-1");
    assert_eq!(evaluate_text("(1+i)*(1-i)").unwrap().display(), "2");
    assert_eq!(evaluate_text("abs(3+4i)").unwrap().display(), "5");
    assert_eq!(evaluate_text("conj(2+3i)").unwrap().display(), "2 - 3i");
}

#[test]
fn evaluates_units_and_conversions() {
    assert_eq!(
        evaluate_text("2\"m\" + 30\"cm\"").unwrap().display(),
        "2.3 m"
    );
    assert_eq!(
        evaluate_text("1\"m\" + 1\"cm\"").unwrap().display(),
        "1.01 m"
    );
    assert_eq!(
        evaluate_text("(2\"m\") / (2\"s\")").unwrap().display(),
        "1 m/s"
    );
    assert_eq!(
        evaluate_text("20\"°C\" + 5\"K\"").unwrap().display(),
        "25°C"
    );
}

#[test]
fn evaluates_matrices_and_advanced_helpers() {
    assert_eq!(
        evaluate_text("((1,2),(3,4))").unwrap().display(),
        "[1, 2; 3, 4]"
    );
    assert_eq!(evaluate_text("det([1,2;3,4])").unwrap().display(), "-2");
    assert_eq!(
        evaluate_text("inv([1,2;3,4])").unwrap().display(),
        "[-2, 1; 1.5, -0.5]"
    );
    assert_eq!(
        evaluate_text("solve([2,1;1,3],[1;2])").unwrap().display(),
        "[0.2; 0.6]"
    );
    assert_eq!(
        evaluate_text("range(1, 5)").unwrap().display(),
        "[1, 2, 3, 4, 5]"
    );
    assert_eq!(
        evaluate_text("reshape(range(1,6), 2, 3)")
            .unwrap()
            .display(),
        "[1, 2, 3; 4, 5, 6]"
    );
    assert_eq!(
        evaluate_text("dot([1,2,3],[4,5,6])").unwrap().display(),
        "32"
    );
    assert_eq!(
        evaluate_text("cross([1,0,0],[0,1,0])").unwrap().display(),
        "[0, 0, 1]"
    );
    assert_eq!(evaluate_text("norm([3,4])").unwrap().display(), "5");
    assert_eq!(evaluate_text("rank([1,2;3,4])").unwrap().display(), "2");
    assert_eq!(
        evaluate_text("eigenvalues([2,1;1,2])").unwrap().display(),
        "[1, 3]"
    );
}

#[test]
fn evaluates_variables_functions_and_documents() {
    let items = vec![
        ("a".to_string(), "a := 2".to_string()),
        ("function".to_string(), "f(x) := x^2 + a".to_string()),
        ("use".to_string(), "f(3)".to_string()),
    ];
    assert_eq!(value_of(&items, "use").as_deref(), Some("11"));
}

#[test]
fn detects_cycles_and_duplicates() {
    let items = vec![
        ("a".to_string(), "a := b".to_string()),
        ("b".to_string(), "b := a".to_string()),
        ("dup".to_string(), "a := 3".to_string()),
    ];
    let results = evaluate_document(&items);
    assert!(matches!(results[0].result, Err(EvalError::DependencyCycle)));
    assert!(matches!(
        results[2].result,
        Err(EvalError::DuplicateDefinition(_))
    ));
}

#[test]
fn evaluates_conditionals_and_loops() {
    assert_eq!(evaluate_text("if(2 < 3, 10, 20)").unwrap().display(), "10");
    assert_eq!(
        evaluate_text("if(1 == 1, text(\"OK\"), text(\"NO\"))")
            .unwrap()
            .display(),
        "OK"
    );
    assert_eq!(
        evaluate_text("for(i, 1, 5, i^2)").unwrap().display(),
        "[1, 4, 9, 16, 25]"
    );
    assert_eq!(
        evaluate_text("for(v, [1,2,3], v^2)").unwrap().display(),
        "[1, 4, 9]"
    );
    assert_eq!(
        evaluate_text("sum(i, 1, 100, i)").unwrap().display(),
        "5050"
    );
    assert_eq!(
        evaluate_text("product(v, [1,2,3,4], v)").unwrap().display(),
        "24"
    );
    assert_eq!(
        evaluate_text("line(x := 0, while(x < 5, x := x + 1), x)")
            .unwrap()
            .display(),
        "5"
    );
}

#[test]
fn rejects_runaway_loops() {
    assert!(evaluate_text("for(i, 1, 200000, i)").is_err());
    assert!(evaluate_text("while(1 < 2, 1)").is_err());
}

#[test]
fn evaluates_symbolic_and_calculus_functions() {
    assert_eq!(
        evaluate_text("simplify(x + 0 + x)").unwrap().display(),
        "2 * x"
    );
    assert_eq!(
        evaluate_text("expand((x+1)^2)").unwrap().display(),
        "x ^ 2 + 2 * x + 1"
    );
    assert_eq!(
        evaluate_text("factor(x^2 - 4)").unwrap().display(),
        "(x - 2) * (x + 2)"
    );
    assert_eq!(
        evaluate_text("diff(x^3 + 2*x, x)").unwrap().display(),
        "3 * x ^ 2 + 2"
    );
    assert_eq!(
        evaluate_text("integrate(x^2, x)").unwrap().display(),
        "x ^ 3 / 3"
    );
    let integral = evaluate_text("integral(sin(x), x, 0, pi)").unwrap();
    assert!((integral.scalar().unwrap() - 2.0).abs() < 1e-6);
    assert_eq!(
        evaluate_text("solve(x^2 - 5*x + 6, x)").unwrap().display(),
        "x = 2, x = 3"
    );
    assert_eq!(
        evaluate_text("polyroots([1, 0, 1])").unwrap().display(),
        "x = i, x = -i"
    );
}

#[test]
fn evaluates_custom_constants_and_builtin_constants() {
    assert_eq!(evaluate_text("gn").unwrap().display(), "9.80665 m/s²");
    assert_eq!(
        evaluate_text("Esteel").unwrap().display(),
        "200000000000 Pa"
    );
    let items = vec![("x".to_string(), "rhocustom * 2".to_string())];
    let constants = vec![("rhocustom".to_string(), "1000 kg/m^3".to_string())];
    let results = evaluate_document_with_constants(&items, &constants);
    assert_eq!(results[0].result.as_ref().unwrap().display(), "2000 kg/m^3");
}

#[test]
fn evaluates_custom_units() {
    let items = vec![
        (
            "unit".to_string(),
            "unit(mi_pulgada) := 2.54 cm".to_string(),
        ),
        (
            "unit2".to_string(),
            "unit(mi_pie) := 12 mi_pulgada".to_string(),
        ),
        ("use".to_string(), "(1 mi_pie) / (1 cm)".to_string()),
    ];
    assert_eq!(value_of(&items, "use").as_deref(), Some("30.48"));
}

#[test]
fn reports_errors_for_invalid_operations() {
    assert!(matches!(
        evaluate_text("1/0").unwrap_err(),
        EvalError::DivisionByZero
    ));
    assert!(evaluate_text("unknown_symbol + 1").is_err());
    assert!(evaluate_text("det([1,2;3,4;5,6])").is_err());
    assert!(evaluate_text("error(\"fallo\")").is_err());
}
