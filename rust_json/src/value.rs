#[derive(Debug, PartialEq, Clone)]
pub enum JsonValue<'a> {
    Null,
    Boolean(bool),
    Number(f64),
    String(&'a str),
    Array(Vec<JsonValue<'a>>),
    Object(std::collections::HashMap<&'a str, JsonValue<'a>>),
}

impl<'a> JsonValue<'a> {
    pub fn stringify(&self) -> String {
        match self {
            JsonValue::Null => "null".to_string(),
            JsonValue::Boolean(b) => b.to_string(),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::String(s) => format!("\"{}\"", s),
            JsonValue::Array(arr) => {
                let elements: Vec<String> = arr.iter()
                    .map(|v| v.stringify())
                    .collect();

                format!("[{}]", elements.join(","))
            },
            JsonValue::Object(obj) => {
                let pairs: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("\"{}\":{}", k, v.stringify()))
                    .collect();

                format!("{{{}}}", pairs.join(","))
            },
        }
    }

    pub fn to_json_string(&self) -> String {
        let mut buf = String::new();
        self.serialize(&mut buf);
        buf
    }

    fn serialize(&self, buf: &mut String) {
        match self {
            JsonValue::Null => buf.push_str("null"),
            JsonValue::Boolean(b) => buf.push_str(&b.to_string()),
            JsonValue::Number(n) => buf.push_str(&n.to_string()),
            JsonValue::String(s) => {
                buf.push('"');
                buf.push_str(s);
                buf.push('"');
            },
            JsonValue::Array(arr) => {
                buf.push('[');
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 { buf.push(',');}
                    val.serialize(buf);
                }
                buf.push(']');
            },
            JsonValue::Object(obj) => {
                buf.push('{');
                for (i, (key, val)) in obj.iter().enumerate() {
                    if i > 0 { buf.push(',');}
                    buf.push('"');
                    buf.push_str(key);
                    buf.push_str("\":");
                    val.serialize(buf);
                }
                buf.push('}');
            },
        }
    }

    pub fn to_pretty_string(&self) -> String {
        let mut buf = String::new();
        self.serialize_pretty(&mut buf, 0);
        buf
    }

    fn serialize_pretty(&self, buf: &mut String, depth: usize) {
        let indent = "  ".repeat(depth);
        let child_indent = "  ".repeat(depth + 1);

        match self {
            JsonValue::Null => buf.push_str("null"),
            JsonValue::Boolean(b) => buf.push_str(&b.to_string()),
            JsonValue::Number(n) => buf.push_str(&n.to_string()),
            JsonValue::String(s) => {
                buf.push('"');
                buf.push_str(s);
                buf.push('"');
            },
            JsonValue::Array(arr) => {
                if arr.is_empty() {
                    buf.push_str("[]");
                } else {
                    buf.push_str("[\n");
                    for (i, val) in arr.iter().enumerate() {
                        if i > 0 { buf.push_str(",\n");}
                        buf.push_str(&child_indent);
                        val.serialize_pretty(buf, depth + 1);
                    }
                    buf.push_str("\n");
                    buf.push_str(&indent);
                    buf.push(']');
                }
            },
            JsonValue::Object(obj) => {
                if obj.is_empty() {
                    buf.push_str("{}");
                } else {
                    buf.push_str("{\n");
                    for (i, (key, val)) in obj.iter().enumerate() {
                        if i > 0 { buf.push_str(",\n");}
                        buf.push_str(&child_indent);
                        buf.push('"');
                        buf.push_str(key);
                        buf.push_str("\": ");
                        val.serialize_pretty(buf, depth + 1);
                    }
                    buf.push_str("\n");
                    buf.push_str(&indent);
                    buf.push('}');
                }
            },
        }
    }
}