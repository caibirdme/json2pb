use super::{
    parser,
    Result,
};

pub fn visit_json_root(v: &parser::JsonValue) -> Result<Message> {
    if let parser::JsonValue::Object(h) = v {
        let mut obj = Message::new();
        for (k,v) in h {
            obj.push(ObjField{
                field_type: visit_json_ele(v)?,
                field_name: k.clone(),
            })
        }
        Ok(obj)
    } else {
        Err("root object must be an object".into())
    }
}

fn visit_json_ele(v: &parser::JsonValue) -> Result<BaseValue> {
    match v {
        &parser::JsonValue::Num(num) => {
            Ok(BaseValue::Scalar(parse_num(num)))
        },
        parser::JsonValue::Boolean(_) => {
            Ok(BaseValue::Scalar(ScalarValue::Bool))
        },
        parser::JsonValue::Str(_) => {
            Ok(BaseValue::Scalar(ScalarValue::Str))
        },
        parser::JsonValue::Array(arr) => {
            if arr.is_empty() {
                Err("cannot inference element's type of array".into())
            } else {
                let ele_type = visit_list_ele(&arr[0])?;
                Ok(BaseValue::List(ele_type))
            }
        },
        parser::JsonValue::Object(obj) => {
            Ok(BaseValue::Message(parse_object(obj)?))
        },
    }
}

const MAX_SAFE_INTEGER: i64 = 1<<52-1;

fn parse_num(num:f64) -> ScalarValue {
    if (num.floor()-num).abs() > f64::EPSILON {
        return ScalarValue::Double;
    }
    let v = num.floor() as i64;
    if v <= MAX_SAFE_INTEGER && v >= -MAX_SAFE_INTEGER {
        ScalarValue::Int64
    } else {
        ScalarValue::Double
    }
}

fn parse_object(obj: &Vec<(String, parser::JsonValue)>) -> Result<Message> {
    if obj.is_empty() {
        Err("cannot inference object type".into())
    } else {
        let mut obj_type = Message::new();
        for (k,v) in obj {
            obj_type.push(ObjField{
                field_name: k.clone(),
                field_type: visit_json_ele(v)?,
            });
        }
        Ok(obj_type)
    }
}

fn visit_list_ele(v: &parser::JsonValue) -> Result<ListEle> {
    match v {
        parser::JsonValue::Object(obj) => {
            Ok(ListEle::Message(parse_object(obj)?))
        },
        parser::JsonValue::Array(_) => {
            Err("protobuf doesn't support nested array".into())
        },
        &parser::JsonValue::Num(num) => {
            Ok(ListEle::Scalar(parse_num(num)))
        },
        parser::JsonValue::Boolean(_) => {
            Ok(ListEle::Scalar(ScalarValue::Bool))
        },
        parser::JsonValue::Str(_) => {
            Ok(ListEle::Scalar(ScalarValue::Str))
        },
    }
}

#[derive(Eq, PartialEq)]
pub enum ScalarValue {
    Double,
    Int64,
    Bool,
    Str,
}

impl ScalarValue{
    fn to_string(&self) -> String {
        match self {
            ScalarValue::Str => "string",
            ScalarValue::Bool => "bool",
            ScalarValue::Double => "double",
            ScalarValue::Int64 => "int64",
        }.to_owned()
    }
}

pub enum BaseValue {
    Scalar(ScalarValue),
    Message(Message),
    List(ListEle),
}

pub struct ObjField {
    field_type: BaseValue,
    field_name: String,
}

pub enum ListEle {
    Scalar(ScalarValue),
    Message(Message),
}

pub struct Message(Vec<ObjField>);

impl Message {
    fn new() ->Self {
        Self(vec![])
    }
    fn push(&mut self, data: ObjField) {
        self.0.push(data);
    }
    fn gen(&self, buf: &mut IdentBuffer, name: &str) {
        buf.write_with_ident(KW_MESSAGE);
        buf.write(" ");
        buf.write(name);
        buf.writeln(" {");
        buf.add_ident(TAB_SPACE);
        let mut nested_obj = vec![];
        for (seq, field) in self.0.iter().enumerate() {
            // 从1开始
            let seq = seq + 1;
            match &field.field_type {
                BaseValue::Scalar(s) => buf.write_with_ident(s.to_string().as_str()),
                BaseValue::Message(obj) => {
                    let type_name = to_message_name(&field.field_name);
                    nested_obj.push((type_name.clone(), obj));
                    buf.write_with_ident(type_name.as_str());
                },
                BaseValue::List(ele) =>{
                    buf.write_with_ident(KW_REPEATED);
                    buf.write(" ");
                    match ele {
                        ListEle::Scalar(s) => buf.write(s.to_string().as_str()),
                        ListEle::Message(obj) => {
                            let type_name = to_message_name(&field.field_name);
                            nested_obj.push((type_name.clone(), obj));
                            buf.write(type_name.as_str());
                        }
                    }
                }
            }
            buf.write(" ");
            buf.write(field.field_name.as_str());
            buf.write(" = ");
            buf.write(seq.to_string().as_str());
            buf.writeln(";");
        }

        if !nested_obj.is_empty() {
            buf.writeln("");
            for (k,v) in nested_obj {
                v.gen(buf, &k);
            }
        }

        buf.dec_ident(TAB_SPACE);
        buf.writeln_with_ident("}");
    }
}

const KW_MESSAGE: &str = "message";
const KW_REPEATED: &str = "repeated";

const TAB_SPACE: usize = 4;

pub fn gen_pb_def(obj: &Message) -> String {
    let ns = "root_data";
    let mut buf = IdentBuffer::new(0);
    obj.gen(&mut buf, ns);
    buf.to_string()
}

pub struct IdentBuffer {
    ident: usize,
    buf: String,
}

impl IdentBuffer {
    fn new(ident: usize) -> Self {
        Self{
            ident,
            buf: String::new(),
        }
    }
    fn write_ident(&mut self) {
        for _ in 0..self.ident {
            self.buf.push(' ');
        }
    }
    pub fn writeln(&mut self, data: &str) {
        self.write(data);
        self.buf.push('\n');
    }
    pub fn writeln_with_ident(&mut self, data:&str) {
        self.write_with_ident(data);
        self.buf.push('\n');
    }
    pub fn write_with_ident(&mut self, data: &str) {
        self.write_ident();
        self.buf.push_str(data);
    }
    pub fn write(&mut self, data: &str) {
        self.buf.push_str(data);
    }
    pub fn add_ident(&mut self, delta: usize) {
        self.ident += delta;
    }
    pub fn dec_ident(&mut self, delta: usize) {
        if self.ident >= delta {
            self.ident -= delta;
        } else {
            self.ident = 0;
        }
    }
    pub fn to_string(&self) -> String {
        self.buf.clone()
    }
}

const DEFAULT_MSG_NAME: &str = "DefaultMessageName";

fn to_message_name(s: &str) -> String {
    if s.is_empty() {
        return DEFAULT_MSG_NAME.to_string();
    }
    let arr = s.split('_').filter(|v| !v.is_empty()).collect::<Vec<&str>>();
    let mut r = vec![];
    let mut q: Vec<char> = vec![];
    for v in arr {
        q.clear();
        let mut it = v.chars();
        let first_char = it.next().unwrap();
        q.push(first_char.to_uppercase().next().unwrap_or(first_char));
        it.for_each(|v| q.push(v.to_lowercase().next().unwrap_or(v)));
        r.append(&mut q);
    }
    r.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_message_name() {
        let test_cases = vec![
            ("foo", "Foo"),
            ("foo_bar_baz", "FooBarBaz"),
            ("", DEFAULT_MSG_NAME),
            ("FOO_BAR_BAZ", "FooBarBaz"),
            ("foo_4321", "Foo4321"),
            ("foO_123_bAR_4", "Foo123Bar4"),
            ("___foo___bar", "FooBar"),
        ];
        for (s, expect) in test_cases {
            assert_eq!(to_message_name(s), expect);
        }
    }
}