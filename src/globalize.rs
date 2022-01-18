use crate::data::*;

pub fn globalize_module(module: RefalModule) -> RefalModule {
    let name = module.name;
    let functions = module
        .functions
        .into_iter()
        .map(|it| globalize_function(&name, it))
        .collect();
    RefalModule { name, functions }
}

fn globalize_function(module_name: &str, function: Function) -> Function {
    let mut g_name = String::new();
    g_name.push_str(module_name);
    g_name.push('.');
    g_name.push_str(&function.name);
    let sentences = function
        .sentences
        .into_iter()
        .map(|it| globalize_sentence(module_name, it))
        .collect();
    Function {
        name: g_name,
        sentences,
    }
}

fn globalize_sentence(module_name: &str, sentence: Sentence) -> Sentence {
    let pattern = sentence.pattern;
    let expression = globalize_expression(module_name, sentence.expression);
    Sentence {
        pattern,
        expression,
    }
}

fn globalize_expression(module_name: &str, expression: Vec<RefalObject>) -> Vec<RefalObject> {
    let mut result = Vec::<RefalObject>::with_capacity(expression.len());
    let mut prev_fun_br = false;
    for object in expression.into_iter() {
        match object {
            RefalObject::FunBracketL => {
                prev_fun_br = true;
                result.push(object)
            }
            RefalObject::Symbol(ref name) if prev_fun_br => {
                if name.contains('.') {
                    result.push(object)
                } else {
                    let mut globalized = module_name.to_owned();
                    globalized.push('.');
                    globalized.push_str(name);
                    result.push(RefalObject::Symbol(globalized));
                }
                prev_fun_br = false;
            }
            _ => {
                result.push(object);
                prev_fun_br = false;
            }
        }
    }
    result
}
