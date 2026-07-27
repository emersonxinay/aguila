# Guía Maestra de Desarrollo Backend con Águila 

Esta documentación detalla las capacidades del lenguaje Águila para el desarrollo de APIs robustas y escalables, utilizando el framework nativo **Nido**.

---

##  Tabla de Contenidos
1.  [Prerrequisitos (DB)](file:///Users/emersonespinoza/Documents/proyectos/proyecto_nuevo_lenguaje/docs/guia_api_aguila.md#L11)
2.  [Inicio Rápido (CLI & Generadores)](file:///Users/emersonespinoza/Documents/proyectos/proyecto_nuevo_lenguaje/docs/guia_api_aguila.md#L23)
3.  [Relaciones (Foreign Keys)](file:///Users/emersonespinoza/Documents/proyectos/proyecto_nuevo_lenguaje/docs/guia_api_aguila.md#L37)
4.  [Estructura del Proyecto](file:///Users/emersonespinoza/Documents/proyectos/proyecto_nuevo_lenguaje/docs/guia_api_aguila.md#L141)
5.  [Conceptos Core (Manual)](file:///Users/emersonespinoza/Documents/proyectos/proyecto_nuevo_lenguaje/docs/guia_api_aguila.md#L161)
    *   App y Rutas
    *   Controladores
    *   Modelos
    *   Middleware
6.  [Desafíos de Ingeniería (5 Problemas)](file:///Users/emersonespinoza/Documents/proyectos/proyecto_nuevo_lenguaje/docs/guia_api_aguila.md#L248)
7.  [Comparativa: Águila vs FastAPI](file:///Users/emersonespinoza/Documents/proyectos/proyecto_nuevo_lenguaje/docs/guia_api_aguila.md#L489)
8.  [Roadmap Tecnológico](file:///Users/emersonespinoza/Documents/proyectos/proyecto_nuevo_lenguaje/docs/guia_api_aguila.md#L509)

---

##  Arquitectura del Framework (Nido)

Nido es el framework web por defecto de Águila, inspirado en Express.js y NestJS. Proporciona una estructura clara para enrutamiento, controladores, modelos y middleware.

###  Prerrequisitos (Base de Datos)

⚠️ **Importante:** Águila se conecta a una base de datos PostgreSQL existente. **No instala PostgreSQL ni crea la base de datos principal.**

Antes de empezar:
1.  Instala PostgreSQL.
2.  Crea una base de datos vacía para tu proyecto:
    ```bash
    createdb tienda  # O el nombre que prefieras
    ```
3.  Águila se encargará de **crear las tablas** dentro de esa base de datos.

### ⚡️ Inicio Rápido con CLI (Generadores)

Águila incluye un potente generador de código (`scaffolding`) para acelerar el desarrollo. Puedes crear una API REST completa (Modelo, Controlador, Rutas y Migración SQL) con un solo comando.

**Sintaxis:**
`aguila crear api <NombreRecurso> <campo>:<Tipo>[:opciones]...`

**Ejemplo Real:**
Imagina que quieres crear un recurso `Usuario` con nombre, email único y edad.

```bash
aguila crear api Usuario nombre:Texto email:Texto:unico edad:Entero
```

###  Relaciones entre Tablas (Foreign Keys)

Puedes definir relaciones (Foreign Keys) directamente en el comando CLI usando la opción `ref:Modelo`.

**Sintaxis:**
`campo_id:Entero:ref:ModeloDestino`

**Ejemplo:**
Crear un `Post` que pertenece a un `Usuario`.

```bash
aguila crear api Post titulo:Texto usuario_id:Entero:ref:Usuario
```

Esto hará 3 cosas mágicas:
1.  **SQL:** Agrega `REFERENCES usuarios(id)` en la tabla `posts`.
2.  **Modelo:** Crea un método helper `obtener_usuario()` en `Post.ag` para traer el objeto relacionado fácilmente.
3.  **Validación:** Asegura integridad referencial en la base de datos.

1. **`migraciones/20251211_crear_usuario.ag`** (SQL de tabla)
```aguila
importar nido

fn arriba() {
    sql = "CREATE TABLE usuarios (
    id SERIAL PRIMARY KEY,
    nombre TEXT,
    email TEXT UNIQUE,
    edad INTEGER
    )"
    imprimir("Ejecutando migracion: Creando tabla...")
    DB.ejecutar(sql)
}

fn abajo() {
    sql = "DROP TABLE usuarios"
    DB.ejecutar(sql)
}
```

2. **`modelos/Usuario.ag`** (Lógica de Datos)
```aguila
importar nido

clase Usuario hereda nido.Modelo {
    fn iniciar() {
        este.tabla_nombre = "usuarios"
        este.columnas = ["nombre", "email", "edad"]
        este.reglas = {
            "email": "unico"
        }
    }
    # Getters/Setters automáticos...
}
```

3. **`controladores/UsuarioControlador.ag`** (Lógica de Negocio)
```aguila
importar json
importar modelos/Usuario

clase UsuarioControlador {
    fn listar(req, res) {
        m = Usuario()
        res.json(200, m.todos())
    }
    
    fn crear(req, res) {
        body = json.parse(req.cuerpo())
        m = Usuario()
        m.nombre(body.get("nombre"))
        m.email(body.get("email"))
        m.edad(body.get("edad"))
        m.guardar()
        res.json(201, m.campos)
    }
}
```

### ‍♂️ Cómo hacerlo funcionar ("Zero Config")

Una vez generados los archivos, solo necesitas 2 pasos para tener tu API funcionando:

**Paso 1: Ejecutar Migración**
Corre el script de migración para crear la tabla en tu base de datos (asegúrate de tener `main.ag` o un script que conecte a DB):
*(Pro-tip: Puedes importar y ejecutar `arriba()` temporalmente en tu main o usar `aguila migrar` si está configurado)*

**Paso 2: Registrar en `main.ag`**
Abre tu archivo principal y agrega estas 2 líneas:

```aguila
desde controladores/UsuarioControlador importar UsuarioControlador

# ... (conexión a DB app = nido.App(DB)) ...

ctrl = UsuarioControlador()
app.post("/usuarios", ctrl.crear)
app.get("/usuarios", ctrl.listar)

app.escuchar(3000, app.rutas)
```

¡Listo! Ya puedes hacer `POST /usuarios` y `GET /usuarios`.

### Estructura de Proyecto Recomendada
```
/mi-api
  /controladores
    UsuarioControlador.ag
    ProductoControlador.ag
  /modelos
    Usuario.ag
    Producto.ag
  /middlewares
    Auth.ag
    Logger.ag
  /utils
    Jwt.ag
  main.ag  <-- Punto de entrada
  nido.ag  <-- Core del framework
```

---

##  Conceptos Core

### 1. Servidor y Enrutamiento (`App`)
La clase `App` inicializa el servidor y gestiona las rutas.

```aguila
importar nido
importar db

# Conexión DB
DB = db.conectar("postgres://user:pass@localhost/db")

app = nido.App(DB)

# Middleware Global
app.usar(auth_middleware)

# Rutas
app.get("/usuarios", usuario_ctrl.listar)
app.post("/usuarios", usuario_ctrl.crear)
app.put("/usuarios", usuario_ctrl.actualizar) # Requiere body con ID
app.delete("/usuarios", usuario_ctrl.borrar)  # Requiere query param ?id=1

# Iniciar
app.escuchar(3000, app.rutas)
```

### 2. Controladores
Los controladores manejan la lógica de negocio. Reciben `req` (Request) y `res` (Response).

```aguila
clase UsuarioControlador {
    fn crear(req, res) {
        # Parsear JSON del cuerpo
        body = json.parse(req.cuerpo())
        
        # Instanciar Modelo
        u = Usuario(req.db)
        u.nombre(body.get("nombre"))
        u.email(body.get("email"))
        
        # Guardar en DB
        u.guardar()
        
        res.json(201, u.campos)
    }
}
```

### 3. Modelos (ORM Active Record Simplificado)
Los modelos abstraen la interacción con SQL. Soportan métodos fluidos (`fluent interface`).

```aguila
clase Usuario hereda Modelo {
    fn iniciar(db_conn) {
        # Configuración de tabla y campos
        super.iniciar(db_conn, "usuarios", ["id", "nombre", "email", "password"])
    }
    
    # Setters fluidos
    fn nombre(v) { este.set("nombre", v); retorna este }
    fn email(v) { este.set("email", v); retorna este }
}
```

### 4. Middleware
Funciones que interceptan requests antes de llegar al controlador. Útil para Auth, Logging, CORS.

```aguila
fn logger_middleware(req, res) {
    imprimir("[LOG] " + req.metodo() + " " + req.url())
    retorna verdadero # Continuar
}

fn auth_middleware(req, res) {
    token = req.header("Authorization")
    si no token {
        res.json(401, {"error": "No autorizado"})
        retorna falso # Detener request
    }
    # ... validar token ...
    retorna verdadero
}
```

---

##  Desafíos de Ingeniería (Google Style en Águila)

A continuación, 5 problemas de entrevista técnica resueltos utilizando Águila, demostrando su capacidad para lógica algorítmica y estructuras de datos.

### Problema 1: Rate Limiter (Limitador de Velocidad)
**Objetivo:** Implementar un middleware que limite a un usuario a 5 peticiones cada 10 segundos.

```aguila
# Almacén en memoria (Global)
LIMITADOR = {}

fn rate_limiter(req, res) {
    ip = req.ip_remota() # Asumimos que request tiene IP
    ahora = reloj() # Tiempo actual en segundos (float)
    
    si no LIMITADOR.existe(ip) {
        LIMITADOR[ip] = []
    }
    
    historico = LIMITADOR[ip]
    # Filtrar peticiones antiguas (> 10s)
    nuevo_historico = []
    para t en historico {
        si (ahora - t) < 10.0 {
            nuevo_historico.agregar(t)
        }
    }
    
    si nuevo_historico.longitud() >= 5 {
        res.json(429, {"error": "Demasiadas peticiones"})
        retorna falso
    }
    
    nuevo_historico.agregar(ahora)
    LIMITADOR[ip] = nuevo_historico
    retorna verdadero
}
```

### Problema 2: Caché LRU (Least Recently Used)
**Objetivo:** Diseñar una estructura de caché de tamaño fijo que descarte el elemento menos usado recientemente.

```aguila
clase LRUCache {
    fn iniciar(capacidad) {
        este.capacidad = capacidad
        este.cache = {} # Mapa clave -> valor
        este.uso = []   # Lista de claves ordenadas por uso (0 = menos usado)
    }

    fn get(clave) {
        si este.cache.existe(clave) {
            # Actualizar uso: Mover al final (más reciente)
            este._actualizar_uso(clave)
            retorna este.cache[clave]
        }
        retorna nulo
    }

    fn put(clave, valor) {
        si este.cache.existe(clave) {
            este.cache[clave] = valor
            este._actualizar_uso(clave)
        } sino {
            si este.uso.longitud() >= este.capacidad {
                # Eliminar menos usado (índice 0)
                menos_usado = este.uso[0]
                este.uso.eliminar(0) 
                este.cache.eliminar(menos_usado) # Falta metodo eliminar en diccionarios v1?
                # Workaround: Recrear diccionario (costoso pero funcional en MVP)
                # En v2.7 se recomienda implementar Diccionario.eliminar nativo.
            }
            este.cache[clave] = valor
            este.uso.agregar(clave)
        }
    }

    fn _actualizar_uso(clave) {
        # Buscar y mover al final
        idx = -1
        i = 0
        para k en este.uso {
            si k == clave { idx = i; romper }
            i = i + 1
        }
        si idx != -1 {
            este.uso.eliminar(idx)
            este.uso.agregar(clave)
        }
    }
}
```

### Problema 3: Validar Paréntesis (Stack)
**Objetivo:** Validar si una cadena de JSON tiene los paréntesis/llaves balanceados.

```aguila
fn validar_parentesis(s) {
    pila = []
    mapa = { ")": "(", "}": "{", "]": "[" }
    
    chars = s.caracteres() # Asumiendo metodo extension chars
    # O iterar por indice si Strings tienen acceso aleatorio
    l = s.longitud()
    i = 0
    
    mientras i < l {
        c = s.subcadena(i, 1) # Obtener char
        
        si (c == "(") o (c == "{") o (c == "[") {
            pila.agregar(c)
        } sino si (c == ")") o (c == "}") o (c == "]") {
            si pila.longitud() == 0 { retorna falso }
            
            ultimo = pila[pila.longitud() - 1]
            esperado = mapa[c]
            
            si ultimo != esperado { retorna falso }
            pila.eliminar(pila.longitud() - 1)
        }
        i = i + 1
    }
    
    retorna pila.longitud() == 0
}
```

### Problema 4: Merge Intervals (Intervalos de Disponibilidad)
**Objetivo:** Dados intervalos de tiempo ocupados `[[1,3], [2,6], [8,10]]`, fusionar los que se solapan. Res: `[[1,6], [8,10]]`.

```aguila
fn merge_intervals(intervalos) {
    # 1. Ordenar por inicio (Asumimos que vienen ordenados o implementar sort)
    # Algoritmo de merge:
    si intervalos.longitud() == 0 { retorna [] }
    
    resultado = [intervalos[0]]
    
    i = 1
    mientras i < intervalos.longitud() {
        actual = intervalos[i]
        ultimo_merge = resultado[resultado.longitud() - 1]
        
        start_act = actual[0]
        end_act = actual[1]
        start_last = ultimo_merge[0]
        end_last = ultimo_merge[1]
        
        si start_act <= end_last {
            # Se solapan, fusionar
            nuevo_end = end_last
            si end_act > end_last { nuevo_end = end_act }
            
            # Actualizar ultimo en resultado (mutable)
            # En Aguila listas son referencia
            resultado[resultado.longitud() - 1] = [start_last, nuevo_end]
        } sino {
            resultado.agregar(actual)
        }
        i = i + 1
    }
    
    retorna resultado
}
```

### Problema 5: K Elementos Más Frecuentes (Top K Logs)
**Objetivo:** Encontrar las `k` IPs más frecuentes en una lista de logs.

```aguila
fn top_k_ips(logs, k) {
    conteo = {}
    
    # 1. Contar frecuencias
    para ip en logs {
        si conteo.existe(ip) {
            conteo[ip] = conteo[ip] + 1
        } sino {
            conteo[ip] = 1
        }
    }
    
    # 2. Ordenar (Bucket Sort simplificado o Selection Sort parcial)
    # Convertir a lista de [ip, count]
    lista = []
    para ip, count en conteo {
        lista.agregar([ip, count])
    }
    
    # Ordenar lista descendente por count (Burbuja simple para demo)
    n = lista.longitud()
    i = 0
    mientras i < k { # Solo necesitamos los top k
        max_idx = i
        j = i + 1
        mientras j < n {
            si lista[j][1] > lista[max_idx][1] {
                max_idx = j
            }
            j = j + 1
        }
        # Swap
        temp = lista[i]
        lista[i] = lista[max_idx]
        lista[max_idx] = temp
        i = i + 1
    }
    
    # 3. Extraer top k
    res = []
    i = 0
    mientras i < k {
        res.agregar(lista[i][0])
        i = i + 1
    }
    retorna res
}
```

---

##  Seguridad y Estabilidad
- **Try/Catch Global:** Águila soporta `intentar/capturar` para evitar caídas del servidor.
- **Validación de Tipos:** Conversión explícita con `entero()`, `decimal()`, `texto()` para seguridad en BD.
- **Inyección de Dependencias:** El objeto `db` se inyecta en cada request, facilitando tests y transacciones.

Esta guía cubre desde la arquitectura básica hasta la resolución de problemas complejos, demostrando que Águila está listo para entrevistas de alto nivel y producción.

---

##  Águila (Nido) vs FastAPI

Una comparación honesta entre el desarrollo con Águila y el estándar de industria actual (Python/FastAPI).

| Característica | Águila (Nido)  | FastAPI (Python)  |
| :--- | :--- | :--- |
| **Filosofía** | **"Baterías Incluidas" (Rails-like).** CLI nativo genera Modelos, Controladores y SQL. Estructura opinionada. | **Microframework.** Tú decides la estructura de carpetas, ORM y validadores. Flexible pero requiere configuración. |
| **Sintaxis** | **Español Nativo.** `mientras`, `si`, `fn`. Ideal para educación y equipos hispanohablantes. | **Inglés / Python Standard.** Universal, pero barrera de entrada para no angloparlantes. |
| **Validación** | **Manual / Helper.** Usas `entero(x)` o reglas en Modelo. | **Automática (Pydantic).** Basada en Type Hints. Muy potente. |
| **Rendimiento** | **Compilado a Bytecode (Rust VM).** Potencialmente más rápido en "hot paths" que Python puro. | **Rápido (Starlette/C).** Muy optimizado para I/O, pero limitado por el GIL de Python en CPU. |
| **Ecosistema** | **Naciente.** Librerías core (HTTP, DB, JSON) incluidas. | **Gigante.** Acceso a todo PyPI (NumPy, Pandas, AI). |
| **Uso Ideal** | Startups rápidas, Educación, Proyectos donde el **Español** es clave. | Sistemas complejos de Enterprise, ML/AI, Integraciones masivas. |

**Conclusión:**
Águila busca la productividad inmediata de herramientas como **Ruby on Rails** o **NestJS** (generadores de código), con la simplicidad de sintaxis de Python, pero en Español. FastAPI es excelente, pero Águila te permite decir: *"¡Mira mamá, programo en español y funciona!"* 

---

##  Roadmap: El Camino para Superar a FastAPI

Para que Águila no solo compita, sino que **supere** a herramientas como FastAPI, el lenguaje debe evolucionar en las siguientes áreas clave. Este es el plan de ingeniería para el futuro de Nido:

### 1. Sistema de Tipos para Validación Automática (El "Pydantic-Killer")
**Estado Actual:** Validación manual (`si body.existe(...)`).
**Meta:** Definir estructuras de datos (DTOs) y que el framework valide automáticamente.

```aguila
# Futuro Código Nido V3
struct CrearUsuarioDTO {
    nombre: Texto,
    email: Texto(formato="email", unico=verdadero),
    edad: Entero(min=18)
}

fn crear(req, body: CrearUsuarioDTO) {
    # 'body' ya viene validado y casteado. Magia.
}
```

### 2. Documentación Interactiva Real (Swagger/OpenAPI)
**Estado Actual:** JSON básico.
**Meta:** Generar especificación OpenAPI 3.0 completa analizando los `structs` de entrada y salida, ofreciendo un panel interactivo (`/docs`) igual o mejor que Swagger UI, totalmente en español.

### 3. Concurrencia Real (Async/Await)
**Estado Actual:** Modelo síncrono (bloqueante).
**Meta:** Implementar un Event Loop (similar a `uvloop` de Python) en la VM de Rust.
- Palabra clave `asincrono fn`.
- I/O no bloqueante para Websockets y miles de conexiones simultáneas.

### 4. Inyección de Dependencias Avanzada
**Estado Actual:** Inyección simple de `db`.
**Meta:** Un contenedor de dependencias que permita inyectar Servicios, Repositorios o Clientes HTTP externos automáticamente en los controladores, facilitando testing y arquitectura hexagonal.

---

Águila ya tiene la base (Sintaxis amigable + VM rápida). Con estas mejoras, se convertirá en la opción #1 no solo por idioma, sino por **potencia técnica**.
