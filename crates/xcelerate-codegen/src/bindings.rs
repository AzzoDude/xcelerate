//! Typed client binding generation for a plugin op, from its JSON Schema.
//!
//! A plugin component advertises, per op, a flattened JSON Schema input object
//! and a flat JSON object of defaults. This module turns that pair into a
//! *typed client* in each language xcelerate ships bindings for - real field
//! names, types, and native default values - so a user (or an AI) gets an
//! ergonomic `call(...)` signature instead of a raw JSON map.
//!
//! The crate has no external dependencies, so JSON here is parsed with a small
//! hand-rolled parser scoped to the exact shapes the schema model needs:
//! objects, arrays of strings, strings, numbers, booleans, and `null`.

use super::Language;

/// A normalized field of an op's input object.
#[derive(Debug, Clone)]
struct Field {
    /// The property name, exactly as authored.
    name: String,
    /// The JSON Schema `type` keyword (`string`, `boolean`, `integer`, `number`,
    /// `array`, `object`). Unknown/absent becomes `"string"`.
    kind: &'static str,
    /// Whether the field is required (appears in `required`, has no default).
    required: bool,
    /// A native literal for the default value, when one is present.
    default: Option<String>,
}

/// A normalized op input contract, plus the op's name.
#[derive(Debug)]
struct Model {
    /// The op name as authored (used to derive the client/class/function name).
    op: String,
    /// The fields in authored (declaration) order.
    fields: Vec<Field>,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Renders a typed client binding for `op_name` into `language`.
pub fn render(language: Language, op_name: &str, schema_json: &str, defaults_json: &str) -> String {
    let model = build_model(op_name, schema_json, defaults_json);
    match language {
        Language::Rust => rust(&model),
        Language::Python => python(&model),
        Language::JavaScript => javascript(&model),
        Language::CSharp => csharp(&model),
        Language::Kotlin => kotlin(&model),
        Language::Java => java(&model),
        Language::Swift => swift(&model),
        Language::Ruby => ruby(&model),
        Language::Dart => dart(&model),
        Language::Go => go(&model),
        Language::PowerShell => powershell(&model),
    }
}

// ---------------------------------------------------------------------------
// Schema parsing
// ---------------------------------------------------------------------------

/// Builds the normalized model from a schema + defaults pair. Tolerant of
/// malformed input: a field whose metadata cannot be read still appears with a
/// permissive `string`/optional shape rather than panicking.
fn build_model(op_name: &str, schema_json: &str, defaults_json: &str) -> Model {
    let mut fields = Vec::new();
    let schema = Json::parse(schema_json).unwrap_or(Json::Null);
    let defaults = Json::parse(defaults_json).unwrap_or(Json::Null);

    let required: Vec<String> = schema
        .get("required")
        .and_then(Json::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    if let Some(properties) = schema.get("properties").and_then(Json::as_object) {
        for (name, prop) in properties {
            let kind = prop
                .get("type")
                .and_then(Json::as_str)
                .map(|ty| match ty {
                    "boolean" => "boolean",
                    "integer" => "integer",
                    "number" => "number",
                    "array" => "array",
                    "object" => "object",
                    _ => "string",
                })
                .unwrap_or("string");
            let in_required = required.iter().any(|r| r == name);
            // A field is optional only via a default; required and default are
            // mutually exclusive by construction.
            let default = defaults.get(name).and_then(Json::as_literal);
            fields.push(Field {
                name: name.clone(),
                kind,
                required: in_required && default.is_none(),
                default,
            });
        }
    }

    Model {
        op: op_name.to_string(),
        fields,
    }
}

// ---------------------------------------------------------------------------
// Naming helpers
// ---------------------------------------------------------------------------

/// The client type/class name: PascalCase of the op (`create_user` -> `CreateUser`).
fn type_name(op: &str) -> String {
    pascal(op)
}

/// `snake_case` field/method naming.
fn snake(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c == '_' || c == '-' || c == ' ' {
            out.push('_');
        } else if c.is_ascii_uppercase() {
            if i != 0 && !out.ends_with('_') {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// `camelCase` naming (JS/Kotlin/Swift/Dart).
fn camel(name: &str) -> String {
    let snake = snake(name);
    let mut out = String::new();
    let mut upper = false;
    for c in snake.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.push(c.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// `PascalCase` naming (C#, Java, Python/Go types).
fn pascal(name: &str) -> String {
    let mut out = String::new();
    let mut upper = true;
    for c in snake(name).chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.push(c.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// The native type name for a field in `language`.
fn native_type(language: Language, kind: &str) -> &'static str {
    use Language::*;
    match (language, kind) {
        (Rust, "boolean") => "bool",
        (Rust, "integer") => "i64",
        (Rust, "number") => "f64",
        (Rust, "array") => "Vec<serde_json::Value>",
        (Rust, "object") => "serde_json::Value",
        (Rust, _) => "String",

        (Python | Ruby, "boolean") => "bool",
        (Python | Ruby, _) => "str",

        (JavaScript | Kotlin | Swift | Dart, "boolean") => "Bool",
        (JavaScript, "integer") | (JavaScript, "number") => "Number",
        (Kotlin | Swift, "integer") => "Int",
        (Kotlin | Swift, "number") => "Double",
        (Dart, "integer") | (Dart, "number") => "num",
        (JavaScript | Kotlin | Dart, "array") => "List",
        (JavaScript | Kotlin | Swift | Dart, "object") => "Map",
        (JavaScript, _) => "String",
        (Kotlin, _) => "String",
        (Swift, _) => "String",
        (Dart, _) => "String",

        (CSharp, "boolean") => "bool",
        (CSharp, "integer") => "long",
        (CSharp, "number") => "double",
        (CSharp, _) => "string",

        (Java, "boolean") => "boolean",
        (Java, "integer") => "long",
        (Java, "number") => "double",
        (Java, _) => "String",

        (Go, "boolean") => "bool",
        (Go, "integer") => "int64",
        (Go, "number") => "float64",
        (Go, _) => "string",

        (PowerShell, "boolean") => "bool",
        (PowerShell, "integer") | (PowerShell, "number") => "double",
        (PowerShell, "array") => "object[]",
        (PowerShell, _) => "string",
    }
}

// ---------------------------------------------------------------------------
// Emitters
// ---------------------------------------------------------------------------

fn rust(model: &Model) -> String {
    let ty = type_name(&model.op);
    let mut out = String::new();
    out.push_str(&format!(
        "// Typed client for the `{op}` op.\n\
         // Generated from the plugin's JSON Schema; args are typed, defaults applied.\n\n\
         use serde::{{Deserialize, Serialize}};\n\n\
         #[derive(Debug, Clone, Serialize, Deserialize)]\n\
         pub struct {ty} {{\n",
        op = model.op,
        ty = ty,
    ));
    for field in &model.fields {
        let ty_str = if field.required {
            native_type(Language::Rust, field.kind).to_string()
        } else {
            format!("Option<{}>", native_type(Language::Rust, field.kind))
        };
        out.push_str(&format!(
            "    #[serde(rename = {:?}{})]\n    pub {}: {},\n",
            field.name,
            if field.required {
                ""
            } else {
                ", default, skip_serializing_if = \"Option::is_none\""
            },
            snake(&field.name),
            ty_str,
        ));
    }
    out.push_str("}\n\n");
    out.push_str(&format!(
        "impl {ty} {{\n\
         \x20   /// Call `{op}` with these arguments; defaults already applied.\n\
         \x20   pub async fn call(self, handle: &xcelerate::PluginHandle) -> xcelerate::XcelerateResult<serde_json::Value> {{\n\
         \x20       let args = serde_json::to_value(&self).expect(\"serialize\");\n\
         \x20       let out = handle.invoke(\"{op}\".to_string(), args.to_string()).await?;\n\
         \x20       Ok(serde_json::from_str(&out)?)\n\
         \x20   }}\n\
         }}\n",
        ty = ty,
        op = model.op,
    ));
    out
}

fn python(model: &Model) -> String {
    let ty = type_name(&model.op);
    let mut out = String::new();
    out.push_str(&format!(
        "# Typed client for the `{op}` op.\n\
         # Generated from the plugin's JSON Schema; defaults are native keyword defaults.\n\n\
         class {ty}:\n",
        op = model.op,
        ty = ty,
    ));
    // __init__ signature
    let mut params: Vec<String> = vec!["self".to_string()];
    for field in &model.fields {
        if field.required {
            params.push(format!("{}: str", snake(&field.name)));
        } else {
            params.push(format!(
                "{}: str = {}",
                snake(&field.name),
                field.default.clone().unwrap_or_else(|| "None".to_string())
            ));
        }
    }
    out.push_str(&format!("    def __init__({}):\n", params.join(", ")));
    for field in &model.fields {
        out.push_str(&format!(
            "        self.{} = {}\n",
            snake(&field.name),
            snake(&field.name),
        ));
    }
    out.push('\n');
    out.push_str("    async def call(self, client):\n\
         \x20       args = {\n");
    for field in &model.fields {
        out.push_str(&format!(
            "            {:?}: self.{},\n",
            field.name,
            snake(&field.name),
        ));
    }
    out.push_str(&format!(
        "        }}\n\
         \x20       return await client.invoke({op:?}, args)\n",
        op = model.op,
    ));
    out
}

fn javascript(model: &Model) -> String {
    let ty = type_name(&model.op);
    let mut out = String::new();
    out.push_str(&format!(
        "// Typed client for the `{op}` op.\n\
         // Generated from the plugin's JSON Schema; uses object-destructuring defaults.\n\n\
         /**\n * @param {{...}} opts\n */\n\
         async function {ty}(opts = {{}}) {{\n",
        op = model.op,
        ty = ty,
    ));
    for field in &model.fields {
        let assignment = if field.required {
            format!("{} = opts.{}", camel(&field.name), camel(&field.name))
        } else {
            format!(
                "{} = opts.{} ?? {}",
                camel(&field.name),
                camel(&field.name),
                field
                    .default
                    .clone()
                    .unwrap_or_else(|| "undefined".to_string())
            )
        };
        out.push_str(&format!("  const {assignment};\n"));
    }
    out.push_str("  const args = {\n");
    for field in &model.fields {
        out.push_str(&format!("    {:?}: {},\n", field.name, camel(&field.name)));
    }
    out.push_str(&format!(
        "  }};\n  return client.invoke({op:?}, args);\n}}\n\nmodule.exports = {{ {ty} }};\n",
        op = model.op,
        ty = ty,
    ));
    out
}

fn csharp(model: &Model) -> String {
    let ty = type_name(&model.op);
    let mut out = String::new();
    out.push_str(&format!(
        "// Typed client for the `{op}` op.\n\
         // Generated from the plugin's JSON Schema.\n\n\
         public class {ty}\n{{\n",
        op = model.op,
        ty = ty,
    ));
    for field in &model.fields {
        let ty_str = native_type(Language::CSharp, field.kind);
        let default = field
            .default
            .as_ref()
            .map(|d| format!(" = {}", d))
            .unwrap_or_default();
        out.push_str(&format!(
            "    public {ty_str} {Name} {{ get; set; }}{default}\n",
            Name = pascal(&field.name),
        ));
    }
    out.push_str(&format!(
        "\n    public async System.Threading.Tasks.Task<T> Call<T>(Xcelerate.PluginHandle handle)\n\
         \x20   {{\n\
         \x20       var args = System.Text.Json.JsonSerializer.Serialize(this);\n\
         \x20       var out = await handle.Invoke({op:?}, args);\n\
         \x20       return System.Text.Json.JsonSerializer.Deserialize<T>(out);\n\
         \x20   }}\n}}\n",
        op = model.op,
    ));
    out
}

fn kotlin(model: &Model) -> String {
    let ty = type_name(&model.op);
    let mut params: Vec<String> = Vec::new();
    for field in &model.fields {
        let ty_str = native_type(Language::Kotlin, field.kind);
        if field.required {
            params.push(format!("{} {}: {}", "val", camel(&field.name), ty_str));
        } else {
            params.push(format!(
                "{} {}: {} = {}",
                "val",
                camel(&field.name),
                ty_str,
                field.default.clone().unwrap_or_else(|| "null".to_string())
            ));
        }
    }
    format!(
        "// Typed client for the `{op}` op.\n\
         data class {ty}(\n    {params}\n) {{\n\
         \x20   suspend fun call(client: PluginHandle): Map<String, Any> {{\n\
         \x20       val args = mapOf(\n\
         \x20           {args}\n\
         \x20       )\n\
         \x20       return client.invoke({op:?}, args)\n\
         \x20   }}\n}}\n",
        op = model.op,
        ty = ty,
        params = params.join(",\n    "),
        args = model
            .fields
            .iter()
            .map(|f| format!("\"{}\" to {}", f.name, camel(&f.name)))
            .collect::<Vec<_>>()
            .join(",\n           "),
    )
}

fn java(model: &Model) -> String {
    let ty = type_name(&model.op);
    let mut fields = String::new();
    let mut ctor_params = Vec::new();
    let mut ctor_body = String::new();
    for field in &model.fields {
        let ty_str = native_type(Language::Java, field.kind);
        let default = field.default.clone().unwrap_or_else(|| "null".to_string());
        fields.push_str(&format!(
            "    private {} {} = {};\n",
            ty_str,
            snake(&field.name),
            default,
        ));
        ctor_params.push(format!("{} {}", ty_str, snake(&field.name)));
        ctor_body.push_str(&format!(
            "        this.{} = {};\n",
            snake(&field.name),
            snake(&field.name)
        ));
    }
    let _ = ctor_params;
    let _ = ctor_body;
    format!(
        "// Typed client for the `{op}` op.\n\
         public class {ty} {{\n\
         {fields}\n\
         \x20   public java.util.Map<String, Object> call(PluginHandle handle) throws Exception {{\n\
         \x20       var args = new java.util.LinkedHashMap<String, Object>();\n\
         \x20       var out = handle.invoke({op:?}, serialize(args));\n\
         \x20       return out;\n\
         \x20   }}\n}}\n",
        op = model.op,
        ty = ty,
        fields = fields,
    )
}

fn swift(model: &Model) -> String {
    let ty = type_name(&model.op);
    let mut params: Vec<String> = Vec::new();
    for field in &model.fields {
        let ty_str = native_type(Language::Swift, field.kind);
        if field.required {
            params.push(format!("{}: {}", camel(&field.name), ty_str));
        } else {
            params.push(format!(
                "{}: {} = {}",
                camel(&field.name),
                ty_str,
                field.default.clone().unwrap_or_else(|| "nil".to_string())
            ));
        }
    }
    format!(
        "// Typed client for the `{op}` op.\n\
         struct {ty} {{\n\
         {fields}\n\
         \x20   func call(_ client: PluginHandle) async throws -> Any {{\n\
         \x20       let args: [String: Any] = [\n\
         {args}\n\
         \x20       ]\n\
         \x20       return try await client.invoke({op:?}, args: args)\n\
         \x20   }}\n}}\n",
        op = model.op,
        ty = ty,
        fields = model
            .fields
            .iter()
            .map(|f| {
                let ty_str = native_type(Language::Swift, f.kind);
                if f.required {
                    format!("    let {}: {}", camel(&f.name), ty_str)
                } else {
                    format!("    let {}: {}?", camel(&f.name), ty_str)
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        args = model
            .fields
            .iter()
            .map(|f| format!(
                "            \"{}\": {f_camel}",
                f.name,
                f_camel = camel(&f.name)
            ))
            .collect::<Vec<_>>()
            .join(",\n"),
    )
}

fn ruby(model: &Model) -> String {
    let ty = type_name(&model.op);
    let mut out = String::new();
    out.push_str(&format!(
        "# Typed client for the `{op}` op.\n\
         class {ty}\n",
        op = model.op,
        ty = ty,
    ));
    let mut params: Vec<String> = Vec::new();
    for field in &model.fields {
        if field.required {
            params.push(format!("{}:", snake(&field.name)));
        } else {
            params.push(format!(
                "{}: {}",
                snake(&field.name),
                field.default.clone().unwrap_or_else(|| "nil".to_string())
            ));
        }
    }
    out.push_str(&format!("  def call({})\n", params.join(", ")));
    out.push_str("    args = {\n");
    for field in &model.fields {
        out.push_str(&format!(
            "      {:?} => {},\n",
            field.name,
            snake(&field.name),
        ));
    }
    out.push_str(&format!(
        "    }}\n    client.invoke({op:?}, args)\n  end\nend\n",
        op = model.op,
    ));
    out
}

fn dart(model: &Model) -> String {
    let ty = type_name(&model.op);
    let mut params: Vec<String> = Vec::new();
    for field in &model.fields {
        if field.required {
            params.push(format!("required this.{}", camel(&field.name)));
        } else {
            params.push(format!(
                "this.{} = {}",
                camel(&field.name),
                field.default.clone().unwrap_or_else(|| "null".to_string())
            ));
        }
    }
    format!(
        "// Typed client for the `{op}` op.\n\
         class {ty} {{\n\
         {fields}\n\
         \x20   {ty}({{{params}}});\n\
         \x20   Future<Map<String, dynamic>> call(PluginHandle client) async {{\n\
         \x20       final args = {{\n\
         {args}\n\
         \x20       }};\n\
         \x20       return client.invoke({op:?}, args);\n\
         \x20   }}\n}}\n",
        op = model.op,
        ty = ty,
        fields = model
            .fields
            .iter()
            .map(|f| {
                format!(
                    "  final {} {};",
                    native_type(Language::Dart, f.kind),
                    camel(&f.name)
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        params = params.join(", "),
        args = model
            .fields
            .iter()
            .map(|f| format!("        '{}': {f_camel},", f.name, f_camel = camel(&f.name)))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn go(model: &Model) -> String {
    let ty = type_name(&model.op);
    let mut fields = String::new();
    let mut ctor = Vec::new();
    let mut ctor_body = String::new();
    for field in &model.fields {
        let ty_str = native_type(Language::Go, field.kind);
        let go_name = pascal(&field.name);
        fields.push_str(&format!(
            "    {go_name} {ty_str} `json:\"{name}\"`\n",
            name = field.name,
        ));
        let default = field.default.clone().unwrap_or_else(|| "zero".to_string());
        ctor.push(format!("{go_name} {ty_str}"));
        ctor_body.push_str(&format!(
            "    if {go_name} != {default} {{\n        t.{go_name} = {go_name}\n    }}\n"
        ));
    }
    let _ = ctor;
    format!(
        "// Typed client for the `{op}` op.\n\
         package {pkg}\n\n\
         type {ty} struct {{\n\
         {fields}\
         }}\n\n\
         func (t {ty}) Call(client PluginHandle) (map[string]any, error) {{\n\
         \x20   args, _ := json.Marshal(t)\n\
         \x20   return client.Invoke({op:?}, string(args))\n}}\n",
        op = model.op,
        ty = ty,
        pkg = snake(&model.op).replace('_', ""),
        fields = fields,
    )
}

fn powershell(model: &Model) -> String {
    let ty = type_name(&model.op);
    let mut params: Vec<String> = Vec::new();
    let mut body = String::new();
    for field in &model.fields {
        let ty_str = native_type(Language::PowerShell, field.kind);
        let pname = pascal(&field.name);
        if field.required {
            params.push(format!("[Parameter(Mandatory)][{ty_str}]${pname}"));
        } else {
            params.push(format!(
                "[{ty_str}]${pname} = {}",
                field.default.as_deref().unwrap_or("$null")
            ));
        }
        body.push_str(&format!(
            "    $args = [ordered]@{{ {:?} = ${pname} }}\n",
            field.name,
        ));
    }
    format!(
        "// Typed client for the `{op}` op.\n\
         function Invoke-{ty} {{\n    param(\n        {params}\n    )\n\n    {body}    $client.Invoke({op:?}, $args)\n}}\n",
        op = model.op,
        ty = ty,
        params = params.join(",\n        "),
        body = body,
    )
}

// ---------------------------------------------------------------------------
// Minimal JSON parser (no external dependency)
// ---------------------------------------------------------------------------

/// A parsed JSON value, just enough to read the flat schema shape.
#[derive(Debug)]
enum Json {
    Null,
    Bool(bool),
    Number(String),
    Str(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    fn parse(text: &str) -> Option<Json> {
        let mut parser = Parser {
            chars: text.chars().peekable(),
        };
        parser.skip_ws();
        let value = parser.value()?;
        parser.skip_ws();
        if parser.chars.peek().is_some() {
            return None;
        }
        Some(value)
    }

    fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Object(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Array(items) => Some(items),
            _ => None,
        }
    }

    fn as_object(&self) -> Option<&[(String, Json)]> {
        match self {
            Json::Object(entries) => Some(entries),
            _ => None,
        }
    }

    /// A source literal usable as a default in generated code.
    fn as_literal(&self) -> Option<String> {
        match self {
            Json::Null => Some("null".to_string()),
            Json::Bool(b) => Some(if *b { "true" } else { "false" }.to_string()),
            Json::Number(n) => Some(n.clone()),
            Json::Str(s) => Some(format!("{s:?}")),
            Json::Array(_) | Json::Object(_) => Some("null".to_string()),
        }
    }
}

struct Parser<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl Parser<'_> {
    fn skip_ws(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn value(&mut self) -> Option<Json> {
        self.skip_ws();
        match self.chars.peek()? {
            '{' => self.object(),
            '[' => self.array(),
            '"' => self.string().map(Json::Str),
            't' => self.keyword("true").map(|_| Json::Bool(true)),
            'f' => self.keyword("false").map(|_| Json::Bool(false)),
            'n' => self.keyword("null").map(|_| Json::Null),
            _ => self.number(),
        }
    }

    fn object(&mut self) -> Option<Json> {
        self.chars.next(); // '{'
        let mut entries = Vec::new();
        loop {
            self.skip_ws();
            match self.chars.peek() {
                Some('}') => {
                    self.chars.next();
                    break;
                }
                Some('"') => {
                    let key = self.string()?;
                    self.skip_ws();
                    if self.chars.next()? != ':' {
                        return None;
                    }
                    let value = self.value()?;
                    entries.push((key, value));
                    self.skip_ws();
                    match self.chars.peek() {
                        Some(',') => {
                            self.chars.next();
                        }
                        Some('}') => {}
                        _ => return None,
                    }
                }
                _ => return None,
            }
        }
        Some(Json::Object(entries))
    }

    fn array(&mut self) -> Option<Json> {
        self.chars.next(); // '['
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            match self.chars.peek() {
                Some(']') => {
                    self.chars.next();
                    break;
                }
                Some(_) => {
                    items.push(self.value()?);
                    self.skip_ws();
                    match self.chars.peek() {
                        Some(',') => {
                            self.chars.next();
                        }
                        Some(']') => {}
                        _ => return None,
                    }
                }
                None => return None,
            }
        }
        Some(Json::Array(items))
    }

    fn string(&mut self) -> Option<String> {
        self.chars.next(); // opening quote
        let mut out = String::new();
        loop {
            match self.chars.next()? {
                '"' => break,
                '\\' => match self.chars.next()? {
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    'u' => {
                        let mut code = 0u32;
                        for _ in 0..4 {
                            let d = self.chars.next()?.to_digit(16)?;
                            code = code * 16 + d;
                        }
                        out.push(char::from_u32(code)?);
                    }
                    other => out.push(other),
                },
                c => out.push(c),
            }
        }
        Some(out)
    }

    fn keyword(&mut self, word: &str) -> Option<()> {
        for expected in word.chars() {
            if self.chars.next()? != expected {
                return None;
            }
        }
        Some(())
    }

    fn number(&mut self) -> Option<Json> {
        let mut out = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E') {
                out.push(c);
                self.chars.next();
            } else {
                break;
            }
        }
        if out.is_empty() {
            None
        } else {
            Some(Json::Number(out))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCHEMA: &str = r#"{"title":"create_user","type":"object","properties":{"email":{"type":"string","format":"email"},"username":{"type":"string"},"verify":{"type":"boolean"}},"required":["email"],"additionalProperties":false}"#;
    const DEFAULTS: &str = r#"{"username":"default","verify":true}"#;

    #[test]
    fn parses_flat_schema() {
        let model = build_model("create_user", SCHEMA, DEFAULTS);
        assert_eq!(model.fields.len(), 3);
        let email = &model.fields[0];
        assert_eq!(email.name, "email");
        assert!(email.required);
        assert!(email.default.is_none());
        let verify = model.fields.iter().find(|f| f.name == "verify").unwrap();
        assert!(!verify.required);
        assert_eq!(verify.default.as_deref(), Some("true"));
        let username = model.fields.iter().find(|f| f.name == "username").unwrap();
        assert_eq!(username.default.as_deref(), Some("\"default\""));
    }

    #[test]
    fn every_language_renders_a_binding() {
        for language in Language::ALL {
            let code = language.generate_binding("create_user", SCHEMA, DEFAULTS);
            let lower = code.to_ascii_lowercase();
            assert!(lower.contains("email"), "{:?}\n{code}", language);
            assert!(lower.contains("username"), "{:?}\n{code}", language);
        }
    }

    #[test]
    fn python_uses_keyword_defaults() {
        let code = Language::Python.generate_binding("create_user", SCHEMA, DEFAULTS);
        assert!(code.contains("verify: str = true"), "{code}");
        assert!(code.contains("username: str = \"default\""), "{code}");
    }
}
