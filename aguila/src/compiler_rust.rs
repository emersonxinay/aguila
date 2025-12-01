use crate::ast::{Expresion, Literal, OperadorBinario, Programa, Sentencia};
use std::collections::HashMap;

/// Compilador que genera Rust code ejecutable directamente
pub struct CompilerRust {
    code: String,
    functions: HashMap<String, String>,
    indent_level: usize,
}

impl CompilerRust {
    pub fn new() -> Self {
        Self {
            code: String::new(),
            functions: HashMap::new(),
            indent_level: 0,
        }
    }

    pub fn compile(&mut self, programa: Programa) -> String {
        // Generar preámbulo con imports y helper functions
        self.code.push_str("use std::time::SystemTime;\n\n");
        self.code.push_str("fn reloj() -> f64 {\n");
        self.code.push_str("    let now = SystemTime::now();\n");
        self.code.push_str(
            "    let since_epoch = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();\n",
        );
        self.code.push_str("    since_epoch.as_secs_f64()\n");
        self.code.push_str("}\n\n");

        self.code.push_str("fn main() {\n");
        self.indent_level = 1;

        // Compilar cada statement
        for sentencia in programa.sentencias {
            self.compile_statement(sentencia);
        }

        self.code.push_str("}\n");

        // Agregar funciones
        for (_, func_code) in &self.functions {
            self.code.push_str(func_code);
            self.code.push_str("\n\n");
        }

        self.code.clone()
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    fn compile_statement(&mut self, stmt: Sentencia) {
        match stmt {
            Sentencia::Imprimir(exprs) => {
                for expr in exprs {
                    let expr_code = self.compile_expression(expr);
                    self.code.push_str(&format!(
                        "{}println!(\"{{}}\", {});\n",
                        self.indent(),
                        expr_code
                    ));
                }
            }
            Sentencia::Asignacion {
                objetivo, valor, ..
            } => {
                let val_code = self.compile_expression(valor);
                match objetivo {
                    Expresion::Identificador(nombre) => {
                        self.code.push_str(&format!(
                            "{}let {} = {};\n",
                            self.indent(),
                            nombre,
                            val_code
                        ));
                    }
                    _ => {}
                }
            }
            Sentencia::Funcion {
                nombre,
                params,
                cuerpo,
                ..
            } => {
                let mut func_code = format!("fn {}(", nombre);
                let params: Vec<String> = params
                    .iter()
                    .map(|p| format!("{}: i32", p.nombre))
                    .collect();
                func_code.push_str(&params.join(", "));
                func_code.push_str(") -> i32 {\n");

                let old_indent = self.indent_level;
                self.indent_level = 1;
                let mut old_code = std::mem::take(&mut self.code);

                for stmt in cuerpo {
                    self.compile_statement(stmt);
                }

                let body = self.code.clone();
                self.code = old_code;
                self.indent_level = old_indent;

                func_code.push_str(&body);
                func_code.push_str("}\n");

                self.functions.insert(nombre, func_code);
            }
            Sentencia::Retornar(expr_opt) => {
                if let Some(expr) = expr_opt {
                    let expr_code = self.compile_expression(expr);
                    self.code
                        .push_str(&format!("{}return {};\n", self.indent(), expr_code));
                } else {
                    self.code.push_str(&format!("{}return 0;\n", self.indent()));
                }
            }
            Sentencia::Si {
                condicion,
                entonces,
                sino,
            } => {
                let cond_code = self.compile_expression(condicion);
                self.code
                    .push_str(&format!("{}if {} {{\n", self.indent(), cond_code));

                self.indent_level += 1;
                for stmt in entonces {
                    self.compile_statement(stmt);
                }
                self.indent_level -= 1;

                if let Some(sino_bloque) = sino {
                    self.code
                        .push_str(&format!("{}}} else {{\n", self.indent()));
                    self.indent_level += 1;
                    for stmt in sino_bloque {
                        self.compile_statement(stmt);
                    }
                    self.indent_level -= 1;
                }

                self.code.push_str(&format!("{}}}\n", self.indent()));
            }
            _ => {
                // Ignorar por ahora
            }
        }
    }

    fn compile_expression(&mut self, expr: Expresion) -> String {
        match expr {
            Expresion::Literal(lit) => match lit {
                Literal::Entero(n) => format!("{}", n),
                Literal::Decimal(n) => format!("{}", n),
                Literal::Texto(s) => format!("\"{}\"", s),
                Literal::Booleano(b) => format!("{}", b),
                Literal::Nulo => "0".to_string(),
            },
            Expresion::Identificador(nombre) => nombre,
            Expresion::Binaria { izq, op, der } => {
                let left = self.compile_expression(*izq);
                let right = self.compile_expression(*der);
                let op_str = match op {
                    OperadorBinario::Suma => "+",
                    OperadorBinario::Resta => "-",
                    OperadorBinario::Multiplicacion => "*",
                    OperadorBinario::Division => "/",
                    OperadorBinario::Menor => "<",
                    OperadorBinario::Mayor => ">",
                    OperadorBinario::Igual => "==",
                    _ => "+", // Fallback
                };
                format!("({} {} {})", left, op_str, right)
            }
            Expresion::Llamada { func, args } => {
                let nombre = match *func {
                    Expresion::Identificador(s) => s,
                    _ => "unknown".to_string(),
                };
                let arg_strs: Vec<String> = args
                    .into_iter()
                    .map(|a| self.compile_expression(a))
                    .collect();
                format!("{}({})", nombre, arg_strs.join(", "))
            }
            _ => "0".to_string(),
        }
    }
}
