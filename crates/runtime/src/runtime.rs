use jtl_parser::{
    lex::Lexer,
    parser::{Arg, Expression, PValue, Parser, Source, Stat},
};
use std::collections::HashMap;

pub type ObjectHashMap = HashMap<String, Value<'static>>;

pub enum Value<'a> {
    Int(u32),
    /// Should return any value but a function (or object for now--lazy to implement atm), because functions aren't supported in the langauge
    Function(&'a dyn Fn(Vec<Value<'_>>) -> Option<Value<'_>>), // NOTE: May chage it to a Result<> with a error message with more infor
    String(String),
    Object(ObjectHashMap),
}

pub struct Context(pub ObjectHashMap);

impl Context {
    pub fn new() -> Self {
        Context(HashMap::new())
    }
    pub fn add_object(&mut self, name: String, object: ObjectHashMap) {
        self.0.insert(name, Value::Object(object));
    }
    pub fn get(&self, key: &str) -> Option<&Value<'_>> {
        self.0.get(key)
    }
}

pub struct Runtime {
    pub global: Context,
}

impl Runtime {
    pub fn new(context: Context) -> Self {
        Runtime { global: context }
    }

    pub fn execute(&self, program: &str) -> Result<String, String> {
        let ast = self.parse_ast(program);
        let buffer = self.process_body(ast.body);

        Ok(buffer.join(""))
    }

    fn process_body(&self, body: Vec<Stat>) -> Vec<String> {
        let mut buffer: Vec<String> = Vec::new();

        for stat in body {
            match stat {
                Stat::Tag {
                    _type,
                    start: _,
                    end: _,
                    value,
                } => {
                    self.process_expression(value, &mut buffer);
                }
                Stat::Text {
                    _type,
                    value,
                    start: _,
                    end: _,
                } => buffer.push(value),
            };
        }

        buffer
    }

    fn process_expression(&self, expression: Expression, buffer: &mut Vec<String>) {
        // NOTE: Should not be None, but can be, because of the parser error-recovery
        let vproperty = expression.property.unwrap();
        if let PValue::Property(property) = vproperty {
            match self.property_type_find_value(property.value) {
                Value::String(s) => {
                    buffer.push(s.to_string());
                    // println!("{}", s)
                }
                // NOTE: P return Function or Obropery function can NOTject (argument can return object but not function)
                Value::Function(func) => {
                    let args = self.collect_arguments1(expression.arguments);
                    let fn_results_pre = func(args);

                    if fn_results_pre.is_none() {
                        buffer.push("(NONE)".to_string());
                    } else {
                        let fn_results = fn_results_pre.unwrap();
                        match fn_results {
                            Value::Int(int) => buffer.push(int.to_string()),
                            Value::String(st) => buffer.push(st),
                            Value::Object(_) => panic!("Unsupported behaver"),
                            Value::Function(_) => panic!("Unexpected behaver"),
                        }
                    }
                }
                _ => panic!("Unsupported property value"),
            }
        }
        // others should be taken care by the parser, TODO: add support for others.. manybe
    }

    fn property_type_find_value(&self, stack: Vec<String>) -> &Value<'_> {
        let mut travarsed: Option<&Value<'_>> = None;

        for st in stack {
            if travarsed.is_none() {
                travarsed = self.global.get(&st);
                continue;
            }

            // NOTE: Not 100% sure what direction to take language at the moment. The language is more function-oriented so far
            // If I decided to make it object-oriented then String and Int should be traversable.
            // But at the moment I'm stiking with function oriented as the future, also seems easier (right now) to develop.

            if let Value::Object(obj) = travarsed.unwrap() {
                travarsed = obj.get(&st);
                continue;
            } else if let Value::String(_) = travarsed.unwrap() {
                panic!("Can't tervarse a string")
            } else if let Value::Int(_) = travarsed.unwrap() {
                panic!("Can't tervarse a Int")
            } else if let Value::Function(_) = travarsed.unwrap() {
                panic!("Can't tervarse a function")
            }
            // match tervarsed.unwrap() {}
        }

        travarsed.unwrap()
    }

    fn collect_arguments1(&self, arguments_pre: Option<Vec<Arg>>) -> Vec<Value<'_>> {
        let mut args = Vec::new();

        if arguments_pre.is_none() {
            return args;
        }
        let arguments = arguments_pre.unwrap();

        for arg in arguments {
            // NOTE: At the moment only single arguments are supported...
            let Arg::Single(data) = arg;
            match data.value {
                PValue::String {
                    _type,
                    start: _,
                    end: _,
                    value,
                } => args.push(Value::String(value)),
                _ => todo!(),
            }
        }

        args
    }

    fn parse_ast(&self, source: &str) -> Source {
        let mut tokenizer = Lexer::from_source(source);
        let tokens_res = tokenizer.scan_tokens();
        if tokens_res.is_err() {
            panic!(
                "\n{}{}\nAt {:?}\n",
                source,
                format!(
                    "{}^ {}",
                    left_pad(tokenizer.pointer - 1, None),
                    tokens_res.err().unwrap()
                ),
                tokenizer.position
            )
        }

        let mut parser = Parser::from_lexer(tokenizer);
        let parse_results = parser.parse();

        if parse_results.errors.len() > 0 {
            let error = &parse_results.errors[0];
            panic!(
                "\n{}{}\n",
                source,
                format!(
                    "{}{} {}",
                    left_pad(error.start.1, None),
                    left_pad(error.end.1 - error.start.1, Some("^")),
                    error.message
                ),
            )
        }

        parse_results.ast
    }
}

fn left_pad(p: usize, char: Option<&str>) -> String {
    let mut res = "".to_string();
    for _ in 0..p {
        res += if char.is_none() { " " } else { char.unwrap() }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_property_runtime() {
        let program = "Hay, {guild.saymore | \"world\"} \n welcome to {guild.name}";
        // let program = "Hello, welcome to \n {guild.name} {guild.saymore | \"World\"}\n";
        let mut context = Context::new();

        let mut guild_object: ObjectHashMap = HashMap::new();
        guild_object.insert("name".to_string(), Value::String("BarFight".to_string()));
        guild_object.insert(
            "saymore".to_string(),
            Value::Function(&|args| {
                if args.len() == 1 {
                    if let Value::String(msg) = &args[0] {
                        return Some(Value::String(format!("{}_sayingmore", msg)));
                    }
                    return None;
                }
                None
            }),
        );

        context.add_object("guild".to_string(), guild_object);
        // println!("{}", || {});
        // Value::Function(&|i| i);

        // context.addObject()
        // context.addInt()
        // context.addString()

        let runtime = Runtime::new(context);

        println!("{:?}", runtime.execute(program))

        // Value => Int | String | Object | Function
        // Object(HashMap<String, Value>)
        // Function => Fn(`args:`Vec<Value>) -> Value

        // let to_placement = |args: Vec<String>| -> String {
        //     if args.len() == 0 {};
        //     "".to_string()
        // };
    }
}
